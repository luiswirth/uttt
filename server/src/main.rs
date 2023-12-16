mod util;

use common::{
  game::{RoundOutcome, RoundState},
  message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
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
        send_message_to_stream(&ServerMessage::SymbolAssignment(curr_player), &mut stream)
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
        self.receive_message(player).unwrap().start_round_request();
      }
    }
  }

  fn play_round(&mut self) -> RoundOutcome {
    println!("New round started.");
    let starting_player: PlayerSymbol = rand::random();
    let mut round_state = RoundState::new(starting_player);

    self
      .broadcast_message(&ServerMessage::RoundStart(starting_player))
      .unwrap();

    // main round loop
    loop {
      if let Some(outcome) = round_state.outcome() {
        return outcome;
      }

      match self.receive_message(round_state.current_player()).unwrap() {
        ClientMessage::PlaceSymbol(chosen_tile) => {
          self
            .send_message(
              &ServerMessage::OpponentPlaceSymbol(chosen_tile),
              round_state.current_player().other(),
            )
            .unwrap();
          round_state.try_play_move(chosen_tile).unwrap();
        }
        ClientMessage::GiveUp => {
          self
            .send_message(
              &ServerMessage::OpponentGiveUp,
              round_state.current_player().other(),
            )
            .unwrap();
          return RoundOutcome::Win(round_state.current_player().other());
        }
        e => panic!("unexpected meessage: `{:?}`", e),
      };
    }
  }

  fn stream_mut(&mut self, player: PlayerSymbol) -> &mut TcpStream {
    &mut self.streams[player.idx()]
  }

  fn receive_message(&mut self, player: PlayerSymbol) -> io::Result<ClientMessage> {
    receive_message_from_stream(self.stream_mut(player))
  }
  fn send_message(&mut self, message: &ServerMessage, player: PlayerSymbol) -> io::Result<()> {
    send_message_to_stream(message, self.stream_mut(player))
  }
  fn broadcast_message(&mut self, message: &ServerMessage) -> io::Result<()> {
    for p in PLAYERS {
      self.send_message(message, p)?;
    }
    Ok(())
  }
}
