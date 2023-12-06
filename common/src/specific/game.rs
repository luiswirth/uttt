use crate::Player;

use super::{
  board::OuterBoard,
  pos::{GlobalPos, InnerPos, OuterPos},
};

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

  pub fn could_place_symbol(&self, global_pos: GlobalPos) -> bool {
    self
      .curr_outer_pos_opt
      .map(|curr_outer_pos| curr_outer_pos == OuterPos::from(global_pos))
      .unwrap_or(true)
      && self.outer_board.could_place_symbol(global_pos)
  }

  pub fn try_place_symbol(&mut self, global_pos: GlobalPos, symbol: Player) -> bool {
    self
      .curr_outer_pos_opt
      .map(|curr_outer_pos| curr_outer_pos == OuterPos::from(global_pos))
      .unwrap_or(true)
      && self.outer_board.try_place_symbol(global_pos, symbol)
  }

  pub fn update_outer_pos(&mut self, last_move_pos: GlobalPos) {
    let next_outer_pos = InnerPos::from(last_move_pos).as_outer();
    self.curr_outer_pos_opt = self
      .outer_board
      .tile(next_outer_pos)
      .board_state()
      .is_placeable()
      .then_some(next_outer_pos);
  }
}
