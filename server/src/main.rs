use common::{
  message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
  GameState, PlayerSymbol, PLAYER_SYMBOLS,
};

use std::net::{TcpListener, TcpStream};

use tracing::{error, info};

pub struct Server {
  streams: [TcpStream; 2],
  game_state: GameState,
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

    let starting_player = rand::random();
    let game_state = GameState::new(starting_player);

    Ok(Self {
      streams,
      game_state,
    })
  }

  fn play_game(&mut self) -> eyre::Result<()> {
    self.broadcast_message(&ServerMessage::GameStart(self.game_state.current_player))?;

    loop {
      if self.game_state.current_inner_board.is_none() {
        info!("receiving choose innerboard");
        let ClientMessage::ChooseInnerBoardProposal(inner_board) =
          self.receive_message(self.game_state.current_player)?
        else {
          panic!("invalid message received");
        };
        // TODO: verify inner board selection
        self.broadcast_message(&ServerMessage::ChooseInnerBoardAccepted(inner_board))?;
      }

      info!("receiving placesymbol");
      let ClientMessage::PlaceSymbolProposal(tile) =
        self.receive_message(self.game_state.current_player)?
      else {
        panic!("invalid message received");
      };
      // TODO: verify tile selection
      self.broadcast_message(&ServerMessage::PlaceSymbolAccepted(tile))?;

      // TODO: check if innerboard occupied
      self.game_state.current_inner_board = Some(tile.as_outer());
      self.game_state.current_player = self.game_state.current_player.other();
    }
  }
}

impl Server {
  pub fn send_message(
    &mut self,
    message: &ServerMessage,
    player: PlayerSymbol,
  ) -> eyre::Result<()> {
    send_message_to_stream(message, &mut self.streams[player.to_idx()])?;
    Ok(())
  }

  pub fn broadcast_message(&mut self, message: &ServerMessage) -> eyre::Result<()> {
    for p in PLAYER_SYMBOLS {
      self.send_message(message, p)?;
    }
    Ok(())
  }

  pub fn receive_message(&mut self, player: PlayerSymbol) -> eyre::Result<ClientMessage> {
    receive_message_from_stream(&mut self.streams[player.to_idx()])
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
