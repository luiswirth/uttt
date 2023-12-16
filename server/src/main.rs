mod util;

use common::{
  game::{PlayerAction, RoundOutcome, RoundState},
  msg::{
    receive_msg_from_stream, send_msg_to_stream, ClientMsgAction, ClientReqRoundStart,
    ServerMsgOpponentAction, ServerMsgRoundStart, ServerMsgSymbolAssignment,
  },
  PlayerSymbol, DEFAULT_SOCKET_ADDR, PLAYERS,
};

use std::{
  io,
  net::{SocketAddrV4, TcpListener, TcpStream},
};

fn main() {
  let mut server = Server::connect();
  server.play_game()
}

pub struct Server {
  /// sorted according to `Player`
  streams: [TcpStream; 2],
}

impl Server {
  pub fn connect() -> Self {
    let socket_addr = match cfg!(feature = "auto_connect") {
      false => {
        let ip_addr = util::read_ip();
        let port = util::read_port();
        SocketAddrV4::new(ip_addr, port)
      }
      true => DEFAULT_SOCKET_ADDR,
    };
    let listener = TcpListener::bind(socket_addr).expect("Failed to bind TcpListener.");

    let mut curr_player: PlayerSymbol = rand::random();

    println!("Waiting for connections...");
    let mut streams: Vec<(PlayerSymbol, TcpStream)> = listener
      .incoming()
      .filter_map(|stream| match stream {
        Ok(s) => Some(s),
        Err(e) => {
          println!("Connecting to TcpStream failed: {}", e);
          println!("Continuing to listen for connections...");
          None
        }
      })
      .take(2)
      .enumerate()
      .map(|(i, mut stream)| {
        send_msg_to_stream(&ServerMsgSymbolAssignment(curr_player), &mut stream)
          .expect("Sending message failed.");
        println!("Player{} connected {}", i, stream.peer_addr().unwrap());

        let r = (curr_player, stream);
        curr_player.switch();
        r
      })
      .collect();

    streams.sort_by_key(|&(s, _)| s);
    let streams = streams
      .into_iter()
      .map(|(_, s)| s)
      .collect::<Vec<_>>()
      .try_into()
      .unwrap();

    Self { streams }
  }

  pub fn play_game(&mut self) {
    // main game loop
    loop {
      let outcome = self.play_round();
      match outcome {
        RoundOutcome::Win(p) => {
          println!("Player {:?} won!", p);
        }
        RoundOutcome::Draw => {
          println!("Draw!");
        }
      }

      for player in PLAYERS {
        let _: ClientReqRoundStart = self.receive_msg(player).unwrap();
      }
    }
  }

  fn play_round(&mut self) -> RoundOutcome {
    println!("New round started.");
    let starting_player: PlayerSymbol = rand::random();
    let mut round_state = RoundState::new(starting_player);

    self
      .broadcast_msg(&ServerMsgRoundStart(starting_player))
      .unwrap();

    // main round loop
    loop {
      if let Some(outcome) = round_state.outcome() {
        return outcome;
      }

      let ClientMsgAction(action) = self.receive_msg(round_state.current_player()).unwrap();

      let opponent_msg = ServerMsgOpponentAction(action);
      self
        .send_msg(&opponent_msg, round_state.current_player().other())
        .unwrap();

      match action {
        PlayerAction::MakeMove(chosen_tile) => round_state
          .try_play_move(round_state.current_player(), chosen_tile)
          .unwrap(),
        PlayerAction::GiveUp => return RoundOutcome::Win(round_state.current_player().other()),
      };
    }
  }

  fn stream_mut(&mut self, player: PlayerSymbol) -> &mut TcpStream {
    &mut self.streams[player.idx()]
  }

  fn receive_msg<Msg: serde::de::DeserializeOwned>(
    &mut self,
    player: PlayerSymbol,
  ) -> io::Result<Msg> {
    receive_msg_from_stream(self.stream_mut(player))
  }
  fn send_msg<Msg: serde::Serialize>(&mut self, msg: &Msg, player: PlayerSymbol) -> io::Result<()> {
    send_msg_to_stream(msg, self.stream_mut(player))
  }
  fn broadcast_msg<Msg: serde::Serialize>(&mut self, msge: &Msg) -> io::Result<()> {
    for p in PLAYERS {
      self.send_msg(msge, p)?;
    }
    Ok(())
  }
}
