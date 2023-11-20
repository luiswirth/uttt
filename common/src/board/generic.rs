use super::{GenericTileBoardState, TrivialTile};

use crate::{line::Line, pos::GenericPos, PlayerSymbol};

/// Inductive type generating the board hierarchy.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GenericBoard<TileType> {
  tiles: [TileType; 9],
  board_state: GenericTileBoardState,
  line_states: [GenericTileBoardState; 8],
}

/// public interface
impl<TileType> GenericBoard<TileType> {
  pub fn tile(&self, pos: impl Into<GenericPos>) -> &TileType {
    let pos = pos.into();
    &self.tiles[pos.linear_idx()]
  }

  pub fn is_free(&self) -> bool {
    self.board_state.is_free()
  }
}

/// private interface
impl<TileType> GenericBoard<TileType> {
  fn tile_mut(&mut self, pos: GenericPos) -> &mut TileType {
    &mut self.tiles[pos.linear_idx()]
  }
  fn line_state(&self, line: Line) -> GenericTileBoardState {
    self.line_states[line.idx()]
  }
  fn line_state_mut(&mut self, line: Line) -> &mut GenericTileBoardState {
    &mut self.line_states[line.idx()]
  }
}

impl GenericBoard<TrivialTile> {
  fn update_super_states(&mut self, local_pos: GenericPos) {
    for line in Line::all_through_point(local_pos) {
      let line_state = line
        .iter()
        .map(|pos| GenericTileBoardState::from(*self.tile(pos)))
        .reduce(|a, b| a.line_combinator(b))
        .unwrap();

      *self.line_state_mut(line) = line_state;
      if line_state.is_occupied_won() {
        self.board_state = line_state;
      }
    }
    if Line::all().all(|line| self.line_state(line).is_unoccupiable_draw()) {
      self.board_state = GenericTileBoardState::UnoccupiableDraw;
    }
  }
}

impl<Innerboard: BoardTrait> GenericBoard<Innerboard> {
  fn update_super_states(&mut self, local_pos: GenericPos) {
    for line in Line::all_through_point(local_pos) {
      let line_state = line
        .iter()
        .map(|pos| self.tile(pos).board_state())
        .reduce(|a, b| a.line_combinator(b))
        .unwrap();

      *self.line_state_mut(line) = line_state;
      if line_state.is_occupied_won() {
        self.board_state = line_state;
      }
    }
    if Line::all().all(|line| self.line_state(line).is_unoccupiable_draw()) {
      self.board_state = GenericTileBoardState::UnoccupiableDraw;
    }
  }
}

/// Inductive trait generating the board hierarchy.
/// This trait only contains the methods, necessary to perform the induction.
pub trait BoardTrait {
  fn board_state(&self) -> GenericTileBoardState;

  /// Returns the trivial tile at the given position, by recursively walking the board hierarchy.
  fn trivial_tile(&self, pos_iter: impl IntoIterator<Item = GenericPos>) -> TrivialTile;

  /// Places a symbol on the given trivial tile, by recursively walking the board hierarchy and
  /// updating the state of the `TrivialTile` and the hierarchy of super states.
  fn place_symbol(&mut self, pos_iter: impl IntoIterator<Item = GenericPos>, symbol: PlayerSymbol);
}

/// `TrivialBoard` is the bottom of the board hierarchy.
/// It is the base case of the inductive type `GenericBoard`.
pub type TrivialBoard = GenericBoard<TrivialTile>;

/// Induction base case.
impl BoardTrait for TrivialBoard {
  fn board_state(&self) -> GenericTileBoardState {
    self.board_state
  }

  fn trivial_tile(&self, pos_iter: impl IntoIterator<Item = GenericPos>) -> TrivialTile {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");
    assert!(pos_iter.next().is_none());

    *self.tile(local_pos)
  }

  fn place_symbol(&mut self, pos_iter: impl IntoIterator<Item = GenericPos>, symbol: PlayerSymbol) {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");
    assert!(pos_iter.next().is_none());

    *self.tile_mut(local_pos) = TrivialTile::new_occupied(symbol);

    self.update_super_states(local_pos);
  }
}

/// Induction over the board hierarchy.
impl<InnerBoard: BoardTrait> BoardTrait for GenericBoard<InnerBoard> {
  fn board_state(&self) -> GenericTileBoardState {
    self.board_state
  }

  fn trivial_tile(&self, pos_iter: impl IntoIterator<Item = GenericPos>) -> TrivialTile {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");

    // recursion
    self.tile(local_pos).trivial_tile(pos_iter)
  }

  fn place_symbol(&mut self, pos_iter: impl IntoIterator<Item = GenericPos>, symbol: PlayerSymbol) {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");

    // recursion
    self.tile_mut(local_pos).place_symbol(pos_iter, symbol);

    self.update_super_states(local_pos);
  }
}
