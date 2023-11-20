use common::{
  board::{BoardTrait, GenericTileBoardState, OuterBoard},
  message::{receive_message_from_stream, send_message_to_stream, ClientMessage, ServerMessage},
  pos::{GlobalPos, InnerPos, OuterPos},
  PlayerSymbol,
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
    let mut outer_board = OuterBoard::default();
    let mut curr_inner_board_pos_opt: Option<OuterPos> = None;

    println!("Game start!");
    println!("{:?} begins.", curr_player);

    // main game loop
    loop {
      println!("---------------------");
      print_outer_board(&outer_board);
      if curr_player == self.this_player {
        println!("Your ({:?}) turn!", curr_player);
      } else {
        println!("Opponents ({:?}) turn!", curr_player);
      }
      let &mut curr_inner_board_pos = curr_inner_board_pos_opt.get_or_insert_with(|| {
        let mut inner_board_pos_sent = None;
        if curr_player == self.this_player {
          println!("Choose InnerBoard.");
          loop {
            let inner_board_pos = OuterPos::new_arr(parse_position());
            if outer_board.tile(inner_board_pos).is_free() {
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
          if outer_board
            .trivial_tile(GlobalPos::from((curr_inner_board_pos, tile_inner_pos)))
            .is_free()
          {
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
      outer_board.place_symbol(
        GlobalPos::from((curr_inner_board_pos, tile_inner_pos_recv)),
        curr_player,
      );

      match outer_board.tile(curr_inner_board_pos).board_state() {
        GenericTileBoardState::FreeUndecided => {}
        GenericTileBoardState::UnoccupiableDraw => {
          println!("InnerBoard {:?} ended in a draw.", curr_inner_board_pos);
        }
        GenericTileBoardState::OccupiedWon(winner) => {
          let player_str = if winner == self.this_player {
            "You"
          } else {
            "Opponent"
          };
          println!("{} won InnerBoard {:?}!", player_str, curr_inner_board_pos);
        }
      }

      match outer_board.board_state() {
        GenericTileBoardState::FreeUndecided => {}
        GenericTileBoardState::UnoccupiableDraw => {
          println!("The game ended in a draw.");
          break Ok(());
        }
        GenericTileBoardState::OccupiedWon(winner) => {
          print_outer_board(&outer_board);
          let player_str = if winner == self.this_player {
            "You"
          } else {
            "Opponent"
          };
          println!("{} won the game!", player_str);
          break Ok(());
        }
      }

      let next_inner_board_pos = tile_inner_pos_recv.as_outer();
      if outer_board.tile(next_inner_board_pos).is_free() {
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
  loop {
    client.play_game()?;
  }
}

pub fn parse_position() -> [u8; 2] {
  // quick and dirty random position
  //let t = std::time::Instant::now().elapsed().as_nanos();
  //return [(t % 3) as u8, (t / 10 % 3) as u8];
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

fn print_outer_board(board: &OuterBoard) {
  for outer_y in 0..3 {
    for inner_y in 0..3 {
      for outer_x in 0..3 {
        for inner_x in 0..3 {
          let global_pos = GlobalPos::new(outer_x * 3 + inner_x, outer_y * 3 + inner_y);
          let outer_pos = OuterPos::from(global_pos);
          let inner_pos = InnerPos::from(global_pos);
          let inner_board = board.tile(outer_pos);
          let c = match inner_board.board_state() {
            GenericTileBoardState::FreeUndecided => match inner_board.tile(inner_pos).0 {
              Some(sym) => match sym {
                PlayerSymbol::Cross => 'X',
                PlayerSymbol::Circle => 'O',
              },
              None => '.',
            },
            GenericTileBoardState::OccupiedWon(sym) => match sym {
              PlayerSymbol::Cross => 'X',
              PlayerSymbol::Circle => 'O',
            },
            GenericTileBoardState::UnoccupiableDraw => '#',
          };
          print!("{}", c);
        }
        print!(" ");
      }
      println!();
    }
    println!();
  }
}
