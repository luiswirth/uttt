use common::{
  generic::board::TileBoardState,
  specific::{
    board::OuterBoard,
    message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
    pos::{GlobalPos, OuterPos},
  },
  Player, PLAYERS,
};

use std::net::{TcpListener, TcpStream};
use tracing::{error, info};

pub struct Server {
  /// sorted according to `Player`
  streams: [TcpStream; 2],
}

impl Server {
  pub fn new() -> eyre::Result<Self> {
    let listener = TcpListener::bind("localhost:42069")?;

    let mut curr_player: Player = rand::random();
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
    let mut curr_inner_board_pos_opt: Option<OuterPos> = None;

    tracing::info!("New game started.");
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

          if outer_board.tile(inner_board_pos).is_free() {
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

        if outer_board
          .trivial_tile(GlobalPos::from((curr_inner_board_pos, tile_inner_pos)))
          .is_free()
        {
          break tile_inner_pos;
        } else {
          self.send_message(&ServerMessage::PlaceSymbolRejected, curr_player)?;
          continue;
        }
      };

      outer_board.place_symbol(
        GlobalPos::from((curr_inner_board_pos, tile_inner_pos)),
        curr_player,
      );
      self.broadcast_message(&ServerMessage::PlaceSymbolAccepted(tile_inner_pos))?;

      match outer_board.tile(curr_inner_board_pos).board_state() {
        TileBoardState::Free => {}
        TileBoardState::Drawn => {
          info!("InnerBoard {:?} ended in a draw.", curr_inner_board_pos);
        }
        TileBoardState::Won(winner) => {
          assert_eq!(winner, curr_player);
          info!("{:?} won InnerBoard {:?}.", winner, curr_inner_board_pos);
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

      let next_inner_board_pos = tile_inner_pos.as_outer();
      if outer_board.tile(next_inner_board_pos).is_free() {
        curr_inner_board_pos_opt = Some(next_inner_board_pos);
      } else {
        curr_inner_board_pos_opt = None;
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

  pub fn receive_message(&mut self, player: Player) -> eyre::Result<ClientMessage> {
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
