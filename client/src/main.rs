use common::{
  message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
  InnerPos, OuterBoard, OuterPos, PlayerSymbol,
};

use std::net::TcpStream;
use tracing::info;

pub struct Client {
  stream: TcpStream,
  this_player: PlayerSymbol,
  _other_player: PlayerSymbol,
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
      _other_player: other_player,
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
      println!("{}", board);
      let &mut curr_inner_board_pos = curr_inner_board_pos_opt.get_or_insert_with(|| {
        let mut inner_board_pos_sent = None;
        if curr_player == self.this_player {
          println!("Your turn!");
          println!("Choose InnerBoard.");
          loop {
            let inner_board_pos = OuterPos::new_arr(parse_position());
            if board.inner_board(inner_board_pos).is_free() {
              self
                .send_message(&ClientMessage::ChooseInnerBoardProposal(inner_board_pos))
                .unwrap();
              inner_board_pos_sent = Some(inner_board_pos);
              break;
            } else {
              println!("InnerBoard is not free. Try again.");
              continue;
            }
          }
        } else {
          println!("Opponents turn!");
        }

        let inner_board_pos_recv = self
          .receive_message()
          .unwrap()
          .choose_inner_board_accepted();
        if let Some(inner_board_pos_sent) = inner_board_pos_sent {
          assert_eq!(inner_board_pos_sent, inner_board_pos_recv)
        } else {
          println!("Opponent chose InnerBoard {:?}.", inner_board_pos_recv);
        };
        inner_board_pos_recv
      });

      let mut tile_inner_pos_sent = None;
      if curr_player == self.this_player {
        println!("Choose Tile inside InnerBoard {:?}.", curr_inner_board_pos);
        loop {
          let tile_inner_pos = InnerPos::new_arr(parse_position());
          if board.tile((curr_inner_board_pos, tile_inner_pos)).is_free() {
            self.send_message(&ClientMessage::PlaceSymbolProposal(tile_inner_pos))?;
            tile_inner_pos_sent = Some(tile_inner_pos);
            break;
          } else {
            println!("Tile is not free. Try again.");
            continue;
          }
        }
      }

      let tile_inner_pos_recv = self.receive_message()?.place_symbol_accepted();
      if let Some(tile_inner_pos_sent) = tile_inner_pos_sent {
        assert_eq!(tile_inner_pos_sent, tile_inner_pos_recv)
      } else {
        println!(
          "Opponent placed symbol at Tile {:?} inside InnerBoard {:?}.",
          tile_inner_pos_recv, curr_inner_board_pos
        );
      }
      board.place_symbol((curr_inner_board_pos, tile_inner_pos_recv), curr_player);

      let next_inner_board_pos = tile_inner_pos_recv.as_outer();
      if board.inner_board(next_inner_board_pos).is_free() {
        curr_inner_board_pos_opt = Some(next_inner_board_pos);
      } else {
        curr_inner_board_pos_opt = None;
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

pub fn parse_position() -> [u8; 2] {
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
