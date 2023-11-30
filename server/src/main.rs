use common::{
  generic::board::TileBoardState,
  specific::{
    board::OuterBoard,
    message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
    pos::{InnerPos, OuterPos},
  },
  Player, DEFAULT_IP, DEFAULT_PORT, PLAYERS,
};

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use tracing::{error, info};

pub struct Server {
  /// sorted according to `Player`
  streams: [TcpStream; 2],
}

impl Server {
  pub fn new() -> eyre::Result<Self> {
    let ip_addr = loop {
      println!(
        "Enter IP address (press enter for default = {}):",
        DEFAULT_IP
      );
      let mut ip_addr = String::new();
      std::io::stdin().read_line(&mut ip_addr)?;
      let ip_addr = ip_addr.trim();

      match ip_addr.is_empty() {
        true => {
          info!("Using default ip address {}", DEFAULT_IP);
          break DEFAULT_IP;
        }
        false => match ip_addr.parse::<Ipv4Addr>() {
          Ok(ip_addr) => break ip_addr,
          Err(e) => {
            error!("Parsing ip address failed: {}", e);
            continue;
          }
        },
      }
    };
    let port = loop {
      println!("Enter port (press enter for default = {}):", DEFAULT_PORT);
      let mut port = String::new();
      std::io::stdin().read_line(&mut port)?;
      let port = port.trim();

      match port.is_empty() {
        true => {
          info!("Using default port {}", DEFAULT_PORT);
          break DEFAULT_PORT;
        }
        false => match port.parse::<u16>() {
          Ok(port) => break port,
          Err(e) => {
            error!("Parsing port failed: {}", e);
            continue;
          }
        },
      }
    };
    let socket_addr = SocketAddrV4::new(ip_addr, port);
    let listener = TcpListener::bind(socket_addr)?;

    let mut curr_player: Player = rand::random();

    info!("Waiting for connections...");
    let mut streams: Vec<(Player, TcpStream)> = listener
      .incoming()
      .filter_map(|stream| match stream {
        Ok(s) => Some(s),
        Err(e) => {
          error!("Connecting to TcpStream failed: {}", e);
          None
        }
      })
      .take(2)
      .enumerate()
      .map(|(i, mut stream)| {
        send_message_to_stream(&ServerMessage::SymbolAssignment(curr_player), &mut stream)
          .expect("sending message failed");
        info!("Player{} connected {}", i, stream.peer_addr().unwrap());
        info!("Player{} was assigned {:?}", i, curr_player);

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

  fn play_game(&mut self) -> eyre::Result<()> {
    let mut outer_board = OuterBoard::default();
    let mut curr_player: Player = rand::random();
    let mut curr_outer_pos_opt: Option<OuterPos> = None;

    tracing::info!("New game started.");
    self.broadcast_message(&ServerMessage::GameStart(curr_player))?;

    // main game loop
    loop {
      let tile_global_pos = self.receive_message(curr_player)?.place_symbol_proposal();
      if let Some(curr_inner_board_pos) = curr_outer_pos_opt {
        if curr_inner_board_pos != OuterPos::from(tile_global_pos) {
          self.send_message(&ServerMessage::PlaceSymbolRejected, curr_player)?;
          panic!("invalid move");
        }
      }
      if !outer_board.tile(OuterPos::from(tile_global_pos)).is_free() {
        self.send_message(&ServerMessage::PlaceSymbolRejected, curr_player)?;
        panic!("invalid move");
      }
      if !outer_board.trivial_tile(tile_global_pos).is_free() {
        self.send_message(&ServerMessage::PlaceSymbolRejected, curr_player)?;
        panic!("invalid move");
      }

      let curr_outer_pos = curr_outer_pos_opt.get_or_insert(OuterPos::from(tile_global_pos));

      outer_board.place_symbol(tile_global_pos, curr_player);
      self.broadcast_message(&ServerMessage::PlaceSymbolAccepted(tile_global_pos))?;
      info!("{:?} placed symbol at {:?}", curr_player, tile_global_pos);

      match outer_board.tile(*curr_outer_pos).board_state() {
        TileBoardState::Free => {}
        TileBoardState::Drawn => {
          info!("InnerBoard {:?} ended in a draw.", curr_outer_pos);
        }
        TileBoardState::Won(winner) => {
          assert_eq!(winner, curr_player);
          info!("{:?} won InnerBoard {:?}.", winner, curr_outer_pos);
        }
      }

      match outer_board.board_state() {
        TileBoardState::Free => {}
        TileBoardState::Drawn => {
          info!("The game ended in a draw.");
          break Ok(());
        }
        TileBoardState::Won(winner) => {
          assert_eq!(winner, curr_player);
          info!("{:?} won the game.", winner);
          break Ok(());
        }
      }

      let next_inner_board_pos = InnerPos::from(tile_global_pos).as_outer();
      if outer_board.tile(next_inner_board_pos).is_free() {
        curr_outer_pos_opt = Some(next_inner_board_pos);
      } else {
        curr_outer_pos_opt = None;
      }

      curr_player = curr_player.other();
    }
  }
}

impl Server {
  pub fn stream_mut(&mut self, player: Player) -> &mut TcpStream {
    &mut self.streams[player.idx()]
  }

  pub fn send_message(&mut self, message: &ServerMessage, player: Player) -> eyre::Result<()> {
    send_message_to_stream(message, self.stream_mut(player))?;
    Ok(())
  }

  pub fn broadcast_message(&mut self, message: &ServerMessage) -> eyre::Result<()> {
    for p in PLAYERS {
      self.send_message(message, p)?;
    }
    Ok(())
  }

  pub fn receive_message(&mut self, player: Player) -> std::io::Result<ClientMessage> {
    receive_message_from_stream(self.stream_mut(player))
  }
}

fn main() -> eyre::Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  let mut server = Server::new()?;
  loop {
    server.play_game()?;
  }
}
