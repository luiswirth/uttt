use common::{
  message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
  OuterBoard, OuterPos, PlayerSymbol, PLAYER_SYMBOLS,
};

use std::net::{TcpListener, TcpStream};
use tracing::{error, info};

pub struct Server {
  /// sorted according to `PlayerSymbol`
  streams: [TcpStream; 2],
}

impl Server {
  pub fn new() -> eyre::Result<Self> {
    let listener = TcpListener::bind("localhost:42069")?;

    let mut curr_symbol: PlayerSymbol = rand::random();
    let mut streams: Vec<(PlayerSymbol, TcpStream)> = listener
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
        send_message_to_stream(&ServerMessage::SymbolAssignment(curr_symbol), &mut stream)
          .expect("sending message failed");
        info!("Player{} connected {}", i, stream.peer_addr().unwrap());
        info!("Player{} was assigned {:?}", i, curr_symbol);

        let r = (curr_symbol, stream);
        curr_symbol.switch();
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
    let mut board = OuterBoard::default();
    let mut curr_player: PlayerSymbol = rand::random();
    let mut curr_inner_board_pos_opt: Option<OuterPos> = None;

    self.broadcast_message(&ServerMessage::GameStart(curr_player))?;

    // main game loop
    loop {
      let &mut curr_inner_board_pos = curr_inner_board_pos_opt.get_or_insert_with(|| {
        // choose inner board
        let inner_board_pos = loop {
          let inner_board_pos = self
            .receive_message(curr_player)
            .unwrap()
            .choose_inner_board_proposal();

          if board.inner_board(inner_board_pos).is_free() {
            break inner_board_pos;
          } else {
            self
              .send_message(&ServerMessage::ChooseInnerBoardRejected, curr_player)
              .unwrap();
            continue;
          }
        };
        self
          .broadcast_message(&ServerMessage::ChooseInnerBoardAccepted(inner_board_pos))
          .unwrap();
        inner_board_pos
      });

      let tile_inner_pos = loop {
        let tile_inner_pos = self.receive_message(curr_player)?.place_symbol_proposal();

        if board.tile((curr_inner_board_pos, tile_inner_pos)).is_free() {
          break tile_inner_pos;
        } else {
          self.send_message(&ServerMessage::PlaceSymbolRejected, curr_player)?;
          continue;
        }
      };

      board.place_symbol((curr_inner_board_pos, tile_inner_pos), curr_player);
      // TODO: check winning conditions
      self.broadcast_message(&ServerMessage::PlaceSymbolAccepted(tile_inner_pos))?;

      let next_inner_board_pos = tile_inner_pos.as_outer();
      if board.inner_board(next_inner_board_pos).is_free() {
        curr_inner_board_pos_opt = Some(next_inner_board_pos);
      } else {
        curr_inner_board_pos_opt = None;
      }

      curr_player = curr_player.other();
    }
  }
}

impl Server {
  pub fn stream_mut(&mut self, player: PlayerSymbol) -> &mut TcpStream {
    &mut self.streams[player.to_idx()]
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
    for p in PLAYER_SYMBOLS {
      self.send_message(message, p)?;
    }
    Ok(())
  }

  pub fn receive_message(&mut self, player: PlayerSymbol) -> eyre::Result<ClientMessage> {
    receive_message_from_stream(self.stream_mut(player))
  }
}

fn main() -> eyre::Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  let mut server = Server::new()?;
  server.play_game()?;

  Ok(())
}
