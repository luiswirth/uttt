use super::{GenericBoard, TrivialTileState};

use crate::{
  pos::{GlobalPos, InnerPos, OuterPos},
  PlayerSymbol,
};

pub type OuterBoard = GenericBoard<InnerBoard>;
impl OuterBoard {
  pub fn inner_board(&self, pos: OuterPos) -> &InnerBoard {
    self.tile(pos.into())
  }
  pub fn trivial_tile(&self, pos: impl Into<GlobalPos>) -> TrivialTileState {
    let pos = pos.into();
    let outer_pos = OuterPos::from(pos);
    let inner_pos = InnerPos::from(pos);
    self.inner_board(outer_pos).trivial_tile(inner_pos)
  }

  /// Places a symbol on the board, updating the state of the tile and all super states (lines and board).
  /// Here (in the case of the outer board) this happens recursively.
  pub fn place_symbol(&mut self, pos: impl Into<GlobalPos>, symbol: PlayerSymbol) {
    assert!(self.board_state().is_free());
    let global_pos = pos.into();
    let outer_pos = OuterPos::from(global_pos);
    let inner_pos = InnerPos::from(global_pos);

    self
      .tile_mut(outer_pos.into())
      .place_symbol(inner_pos, symbol);
    self.update_super_states(outer_pos.into());
  }
}

pub type InnerBoard = GenericBoard<TrivialTileState>;
impl InnerBoard {
  pub fn trivial_tile(&self, pos: InnerPos) -> TrivialTileState {
    *self.tile(pos.into())
  }

  /// Places a symbol on the board, updating the state of the tile and all super states (lines and board).
  /// Here (in the case of the inner board) we are at the bottom of the board hierarchy.
  pub fn place_symbol(&mut self, pos: InnerPos, symbol: PlayerSymbol) {
    assert!(self.board_state().is_free());
    let tile = self.tile_mut(pos.into());
    assert!(tile.is_free());
    *tile = TrivialTileState::new_occupied(symbol);
    self.update_super_states(pos.into());
  }

  pub fn is_free(&self) -> bool {
    self.board_state().is_free()
  }
}
