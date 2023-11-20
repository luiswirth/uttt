use super::{GenericTileBoardState, TrivialTileState};

use crate::{line::Line, pos::GenericPos};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GenericBoard<TileType: TileTrait> {
  tiles: [TileType; 9],
  line_states: [GenericTileBoardState; 8],
  board_state: GenericTileBoardState,
}

impl<TileType: TileTrait> GenericBoard<TileType> {
  pub fn board_state(&self) -> GenericTileBoardState {
    self.board_state
  }
}

// These methods are private, because they are shouldn't be used outside of this module.
// The "tile" terminology is also confusing outside of this module.
// The public interface is implemented specifically on the OuterBoard and InnerBoard types.
impl<TileType: TileTrait> GenericBoard<TileType> {
  pub(super) fn tile(&self, pos: GenericPos) -> &TileType {
    &self.tiles[pos.linear_idx()]
  }
  pub(super) fn tile_mut(&mut self, pos: GenericPos) -> &mut TileType {
    &mut self.tiles[pos.linear_idx()]
  }

  /// updates lines and board state after tile is changed
  pub(super) fn update_super_states(&mut self, changed_tile_pos: GenericPos) {
    for line in Line::all_through_point(changed_tile_pos) {
      let line_state = line
        .iter()
        .map(|pos| self.tile(pos).state())
        .reduce(|a, b| a.line_combinator(b))
        .unwrap();

      self.line_states[line.idx()] = line_state;
      if line_state.is_occupied_won() {
        self.board_state = line_state;
      }
    }
    if Line::all().all(|line| self.line_states[line.idx()].is_unoccupiable_draw()) {
      self.board_state = GenericTileBoardState::UnoccupiableDraw;
    }
  }
}

pub trait TileTrait {
  fn state(&self) -> GenericTileBoardState;
}

impl TileTrait for TrivialTileState {
  fn state(&self) -> GenericTileBoardState {
    match self.0 {
      Some(s) => GenericTileBoardState::OccupiedWon(s),
      None => GenericTileBoardState::FreeUndecided,
    }
  }
}

impl<InnerTile: TileTrait> TileTrait for GenericBoard<InnerTile> {
  fn state(&self) -> GenericTileBoardState {
    self.board_state
  }
}
