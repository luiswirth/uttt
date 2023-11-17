use common::{
  message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
  GameState, InnerPos, OuterPos, PlayerSymbol,
};

use std::net::TcpStream;

use tracing::info;

pub struct Client {
  stream: TcpStream,
  this_player: PlayerSymbol,
  other_player: PlayerSymbol,
}

impl Client {
  pub fn new() -> eyre::Result<Self> {
    println!("Please enter the IP address of the server (e.g. 127.0.0.1):");
    let mut ip_address = String::new();
    std::io::stdin()
      .read_line(&mut ip_address)
      .expect("Failed to read line");
    if ip_address.trim().is_empty() {
      ip_address = String::from("localhost");
    }
    let ip_address = format!("{}:42069", ip_address.trim());
    let mut stream = TcpStream::connect(&ip_address)?;
    info!("Successfully connected to {}", ip_address);

    let message = receive_message_from_stream(&mut stream)?;
    let ServerMessage::SymbolAssignment(this_player) = message else {
      panic!("invalid message received");
    };
    let other_player = this_player.other();
    println!("You are {:?}.", this_player);

    Ok(Self {
      stream,
      this_player,
      other_player,
    })
  }

  pub fn play_game(&mut self) -> eyre::Result<()> {
    let ServerMessage::GameStart(starting_player) = self.receive_message()? else {
      panic!("invalid message received");
    };
    let mut game_state = GameState::new(starting_player);

    println!("Game start!");
    println!("{:?} begins.", starting_player);
    loop {
      if game_state.current_player == self.other_player {
        println!("Opponents move");
        let msg = self.receive_message()?;
        let tile = match msg {
          ServerMessage::ChooseInnerBoardAccepted(inner_board) => {
            game_state.current_inner_board = Some(inner_board);
            println!("InnerBoard {:?} chosen.", inner_board);
            let ServerMessage::PlaceSymbolAccepted(tile) = self.receive_message()? else {
              panic!("invalid message received");
            };
            tile
          }
          ServerMessage::PlaceSymbolAccepted(tile) => tile,
          _ => panic!("invalid message received"),
        };
        game_state.place_symbol(tile, self.other_player);

        // TODO: check if inner board occupied
        game_state.current_inner_board = Some(tile.as_outer());
        game_state.current_player.switch();
      } else {
        println!("Your turn!");
        if game_state.current_inner_board.is_none() {
          println!("Choose InnerBoard (3x3 pos).");
          let mut inner_board = String::new();
          std::io::stdin()
            .read_line(&mut inner_board)
            .expect("Failed to read line");
          let mut inner_board = inner_board.split_whitespace();
          let x = inner_board.next().unwrap().parse().unwrap();
          let y = inner_board.next().unwrap().parse().unwrap();
          let inner_board = OuterPos::new(x, y);
          self.send_message(&ClientMessage::ChooseInnerBoardProposal(inner_board))?;

          // TODO: handle rejection
          let ServerMessage::ChooseInnerBoardAccepted(inner_board_recv) = self.receive_message()?
          else {
            panic!("invalid message received");
          };
          assert_eq!(inner_board, inner_board_recv);
          game_state.current_inner_board = Some(inner_board_recv);
        }

        println!("Choose Tile inside InnerBoard (3x3 pos).");
        let mut tile = String::new();
        std::io::stdin()
          .read_line(&mut tile)
          .expect("failed to read line");
        let mut tile = tile.split_whitespace();
        let x = tile.next().unwrap().parse().unwrap();
        let y = tile.next().unwrap().parse().unwrap();
        let tile = InnerPos::new(x, y);
        self.send_message(&ClientMessage::PlaceSymbolProposal(tile))?;

        // TODO: handle rejection
        let ServerMessage::PlaceSymbolAccepted(tile_recv) = self.receive_message()? else {
          panic!("invalid message received");
        };
        assert_eq!(tile, tile_recv);
        game_state.place_symbol(tile_recv, self.this_player);

        // TODO: check if inner board occupied
        game_state.current_inner_board = Some(tile.as_outer());

        game_state.current_player = game_state.current_player.other();
      }
    }
  }
}

impl Client {
  pub fn send_message(&mut self, msg: &ClientMessage) -> eyre::Result<()> {
    send_message_to_stream(msg, &mut self.stream)
  }

  pub fn receive_message(&mut self) -> eyre::Result<ServerMessage> {
    receive_message_from_stream(&mut self.stream)
  }
}
fn main() -> eyre::Result<()> {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  let mut client = Client::new()?;
  client.play_game()?;

  Ok(())
}
