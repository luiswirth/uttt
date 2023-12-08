use crate::{generic::board::TileBoardState, PlayerSymbol};

use super::{
  board::OuterBoard,
  pos::{GlobalPos, InnerPos, OuterPos},
};

pub struct GameState {
  outer_board: OuterBoard,
  curr_player: PlayerSymbol,
  curr_outer_pos: Option<OuterPos>,
}

impl GameState {
  pub fn new(starting_player: PlayerSymbol) -> Self {
    Self {
      outer_board: OuterBoard::default(),
      curr_player: starting_player,
      curr_outer_pos: None,
    }
  }

  pub fn could_play_move(&self, global_pos: GlobalPos) -> bool {
    self
      .curr_outer_pos
      .map(|curr_outer_pos| curr_outer_pos == OuterPos::from(global_pos))
      .unwrap_or(true)
      && self.outer_board.could_place_symbol(global_pos)
  }

  pub fn try_play_move(&mut self, chosen_tile: GlobalPos) -> bool {
    if self.try_place_symbol(chosen_tile) {
      self.update_outer_pos(chosen_tile);
      self.curr_player.switch();
      true
    } else {
      false
    }
  }

  pub fn board(&self) -> &OuterBoard {
    &self.outer_board
  }
  pub fn current_player(&self) -> PlayerSymbol {
    self.curr_player
  }
  pub fn current_outer_pos(&self) -> Option<OuterPos> {
    self.curr_outer_pos
  }

  pub fn winning_state(&self) -> TileBoardState {
    self.outer_board.board_state()
  }
}

// private methods
impl GameState {
  fn try_place_symbol(&mut self, global_pos: GlobalPos) -> bool {
    self
      .curr_outer_pos
      .map(|curr_outer_pos| curr_outer_pos == OuterPos::from(global_pos))
      .unwrap_or(true)
      && self
        .outer_board
        .try_place_symbol(global_pos, self.curr_player)
  }

  fn update_outer_pos(&mut self, last_move_pos: GlobalPos) {
    let next_outer_pos = InnerPos::from(last_move_pos).as_outer();
    self.curr_outer_pos = self
      .outer_board
      .tile(next_outer_pos)
      .board_state()
      .is_placeable()
      .then_some(next_outer_pos);
  }
}
