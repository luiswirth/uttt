use crate::Player;

use super::{board::OuterBoard, pos::OuterPos};

pub struct GameState {
  pub outer_board: OuterBoard,
  pub curr_player: Player,
  pub curr_outer_pos_opt: Option<OuterPos>,
}

impl GameState {
  pub fn new(starting_player: Player) -> Self {
    Self {
      outer_board: OuterBoard::default(),
      curr_player: starting_player,
      curr_outer_pos_opt: None,
    }
  }
}
