use common::{
  specific::{
    game::GameState,
    message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
  },
  PlayerSymbol, DEBUG, DEFAULT_IP, DEFAULT_PORT, PLAYERS,
};

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};

pub struct Server {
  /// sorted according to `Player`
  streams: [TcpStream; 2],
}

impl Server {
  pub fn connect() -> eyre::Result<Self> {
    let ip_addr = loop {
      println!(
        "Enter IP address (press enter for default = {}):",
        DEFAULT_IP
      );
      let mut ip_addr = String::new();
      if !DEBUG {
        std::io::stdin().read_line(&mut ip_addr)?;
      }
      let ip_addr = ip_addr.trim();

      match ip_addr.is_empty() {
        true => {
          println!("Using default ip address {}", DEFAULT_IP);
          break DEFAULT_IP;
        }
        false => match ip_addr.parse::<Ipv4Addr>() {
          Ok(ip_addr) => break ip_addr,
          Err(e) => {
            println!("Parsing ip address failed: {}", e);
            continue;
          }
        },
      }
    };
    let port = loop {
      println!("Enter port (press enter for default = {}):", DEFAULT_PORT);
      let mut port = String::new();
      if !DEBUG {
        std::io::stdin().read_line(&mut port)?;
      }
      let port = port.trim();

      match port.is_empty() {
        true => {
          println!("Using default port {}", DEFAULT_PORT);
          break DEFAULT_PORT;
        }
        false => match port.parse::<u16>() {
          Ok(port) => break port,
          Err(e) => {
            println!("Parsing port failed: {}", e);
            continue;
          }
        },
      }
    };
    let socket_addr = SocketAddrV4::new(ip_addr, port);
    let listener = TcpListener::bind(socket_addr)?;

    let mut curr_player: PlayerSymbol = rand::random();

    println!("Waiting for connections...");
    let mut streams: Vec<(PlayerSymbol, TcpStream)> = listener
      .incoming()
      .filter_map(|stream| match stream {
        Ok(s) => Some(s),
        Err(e) => {
          println!("Connecting to TcpStream failed: {}", e);
          None
        }
      })
      .take(2)
      .enumerate()
      .map(|(i, mut stream)| {
        send_message_to_stream(&ServerMessage::SymbolAssignment(curr_player), &mut stream)
          .expect("sending message failed");
        println!("Player{} connected {}", i, stream.peer_addr().unwrap());
        println!("Player{} was assigned {:?}", i, curr_player);

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

    Ok(Self { streams })
  }

  fn run(&mut self) {
    loop {
      self.play_game();
    }
  }

  fn play_game(&mut self) {
    let starting_player: PlayerSymbol = rand::random();
    let mut game_state = GameState::new(starting_player);

    self
      .broadcast_message(&ServerMessage::GameStart(starting_player))
      .unwrap();

    // main game loop
    while !game_state.winning_state().is_decided() {
      let tile_global_pos = self
        .receive_message(game_state.current_player())
        .unwrap()
        .place_symbol_proposal();

      if game_state.try_play_move(tile_global_pos) {
        self
          .broadcast_message(&ServerMessage::PlaceSymbolAccepted(tile_global_pos))
          .unwrap();
      } else {
        self
          .broadcast_message(&ServerMessage::PlaceSymbolRejected)
          .unwrap();
        panic!("invalid move! {:?}", tile_global_pos);
      }
    }
  }
}

impl Server {
  pub fn stream_mut(&mut self, player: PlayerSymbol) -> &mut TcpStream {
    &mut self.streams[player.idx()]
  }

  pub fn send_message(
    &mut self,
    message: &ServerMessage,
    player: PlayerSymbol,
  ) -> eyre::Result<()> {
    send_message_to_stream(message, self.stream_mut(player))?;
    Ok(())
  }

  pub fn broadcast_message(&mut self, message: &ServerMessage) -> eyre::Result<()> {
    for p in PLAYERS {
      self.send_message(message, p)?;
    }
    Ok(())
  }

  pub fn receive_message(&mut self, player: PlayerSymbol) -> std::io::Result<ClientMessage> {
    receive_message_from_stream(self.stream_mut(player))
  }
}

fn main() {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  let mut server = Server::connect().unwrap();
  server.run()
}
