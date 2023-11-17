use common::{
  message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
  InnerPos, OuterBoard, OuterPos, PlayerSymbol,
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

    let this_player = receive_message_from_stream::<ServerMessage>(&mut stream)?.symbol_assigment();
    let other_player = this_player.other();
    println!("You are {:?}.", this_player);

    Ok(Self {
      stream,
      this_player,
      other_player,
    })
  }

  pub fn play_game(&mut self) -> eyre::Result<()> {
    let mut curr_player = self.receive_message()?.game_start();
    let mut board = OuterBoard::default();
    let mut curr_inner_board_pos_opt: Option<OuterPos> = None;

    println!("Game start!");
    println!("{:?} begins.", curr_player);

    // main game loop
    loop {
      if curr_player == self.this_player {
        println!("Your turn!");
        let &mut curr_inner_board_pos = curr_inner_board_pos_opt.get_or_insert_with(|| {
          println!("Choose InnerBoard (3x3 pos).");
          let inner_board_pos = OuterPos(parse_pos());
          self
            .send_message(&ClientMessage::ChooseInnerBoardProposal(inner_board_pos))
            // TODO: handle message error
            .unwrap();

          // TODO: handle rejection
          let inner_board_pos_recv = self
            .receive_message()
            // TODO: handle message error
            .unwrap()
            .choose_inner_board_accepted();
          assert_eq!(inner_board_pos, inner_board_pos_recv);
          inner_board_pos_recv
        });

        println!("Choose Tile inside InnerBoard (3x3 pos).");
        let tile_inner_pos = InnerPos(parse_pos());
        self.send_message(&ClientMessage::PlaceSymbolProposal(tile_inner_pos))?;

        // TODO: handle rejection
        let tile_inner_pos_recv = self.receive_message()?.place_symbol_accepted();
        assert_eq!(tile_inner_pos, tile_inner_pos_recv);
        board.place_symbol(
          (curr_inner_board_pos, tile_inner_pos_recv),
          self.this_player,
        );

        // TODO: check if inner board occupied
        curr_inner_board_pos_opt = Some(tile_inner_pos.as_outer());
      } else {
        println!("Opponents move");
        let &mut curr_inner_board_pos = curr_inner_board_pos_opt.get_or_insert_with(|| {
          self
            .receive_message()
            // TODO: handle message error
            .unwrap()
            .choose_inner_board_accepted()
        });
        let tile_inner_pos = self.receive_message()?.place_symbol_accepted();
        board.place_symbol((curr_inner_board_pos, tile_inner_pos), self.other_player);

        let next_inner_board_pos = tile_inner_pos.as_outer();
        if board.get_inner_board(next_inner_board_pos).state.is_free() {
          curr_inner_board_pos_opt = Some(next_inner_board_pos);
        } else {
          curr_inner_board_pos_opt = None;
        }
      }
      curr_player.switch();
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

pub fn parse_pos() -> [u8; 2] {
  loop {
    let mut inner_board = String::new();
    if std::io::stdin().read_line(&mut inner_board).is_err() {
      println!("Failed to read line. Try again.");
      continue;
    }
    match inner_board
      .split_whitespace()
      .map(|s| s.parse::<u8>().ok())
      .map(|op| op.filter(|&x| x < 3))
      .collect::<Option<Vec<_>>>()
      .and_then(|v| v.try_into().ok())
    {
      Some(pos) => return pos,
      None => {
        println!("Invalid input. Try again.");
        continue;
      }
    }
  }
}
