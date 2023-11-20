use crate::PlayerSymbol;

use super::{line::Line, pos::GenericPos};

/// `TrivialBoard` is the bottom of the board hierarchy.
/// It is the base case of the inductive type `GenericBoard`.
pub type TrivialBoard = GenericBoard<TrivialTile>;

/// Inductive type generating the board hierarchy.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GenericBoard<TileType> {
  tiles: [TileType; 9],
  board_state: GenericTileBoardState,
  line_states: [GenericTileBoardState; 8],
}

// reexporting private methods that should be public
// (needed because trait methods can't be private)
#[allow(private_bounds)]
impl<TileType: TileTrait> GenericBoard<TileType> {
  pub fn tile(&self, pos: impl Into<GenericPos>) -> &TileType {
    BoardTrait::tile(self, pos.into())
  }

  pub fn board_state(&self) -> GenericTileBoardState {
    BoardTrait::board_state(self)
  }

  pub fn is_free(&self) -> bool {
    self.board_state.is_free()
  }

  pub fn trivial_tile(&self, pos_iter: impl IntoIterator<Item = GenericPos>) -> TrivialTile {
    BoardTrait::trivial_tile(self, pos_iter.into_iter())
  }
  pub fn place_symbol(&mut self, pos: impl IntoIterator<Item = GenericPos>, symbol: PlayerSymbol) {
    BoardTrait::place_symbol(self, pos.into_iter(), symbol)
  }
}

/// Trait to allow recursion on inductive board hierarchy.
trait BoardTrait {
  type TileType: TileTrait;

  fn tile(&self, pos: GenericPos) -> &Self::TileType;
  fn tile_mut(&mut self, pos: GenericPos) -> &mut Self::TileType;

  fn board_state(&self) -> GenericTileBoardState;
  fn board_state_mut(&mut self) -> &mut GenericTileBoardState;

  fn line_state(&self, line: Line) -> GenericTileBoardState;
  fn line_state_mut(&mut self, line: Line) -> &mut GenericTileBoardState;

  /// Returns the trivial tile at the given position, by recursively walking the board hierarchy.
  fn trivial_tile(&self, mut pos_iter: impl Iterator<Item = GenericPos>) -> TrivialTile {
    let local_pos = pos_iter.next().expect("ran out of positions");
    TileTrait::trivial_tile(self.tile(local_pos), pos_iter)
  }

  /// Places a symbol on the given trivial tile, by recursively walking the board hierarchy and
  /// updating the state of the `TrivialTile` and the hierarchy of super states.
  fn place_symbol(&mut self, mut pos_iter: impl Iterator<Item = GenericPos>, symbol: PlayerSymbol) {
    let local_pos = pos_iter.next().expect("ran out of positions");
    TileTrait::place_symbol(self.tile_mut(local_pos), pos_iter, symbol);
    self.update_super_states(local_pos);
  }

  /// Updates the local super states (line and board states), after a tile at the given pos has changed.
  fn update_super_states(&mut self, local_pos: GenericPos) {
    for line in Line::all_through_point(local_pos) {
      let line_state = line
        .iter()
        .map(|pos| self.tile(pos).tile_state())
        .reduce(|a, b| a.line_combinator(b))
        .unwrap();

      *self.line_state_mut(line) = line_state;
      if line_state.is_occupied_won() {
        *self.board_state_mut() = line_state;
      }
    }
    if Line::all().all(|line| self.line_state(line).is_unoccupiable_draw()) {
      *self.board_state_mut() = GenericTileBoardState::UnoccupiableDraw;
    }
  }
}

/// The only implementation of `BoardTrait` is for `GenericBoard`.
impl<TileType: TileTrait> BoardTrait for GenericBoard<TileType> {
  type TileType = TileType;

  fn tile(&self, pos: GenericPos) -> &Self::TileType {
    &self.tiles[pos.linear_idx()]
  }
  fn tile_mut(&mut self, pos: GenericPos) -> &mut Self::TileType {
    &mut self.tiles[pos.linear_idx()]
  }

  fn board_state(&self) -> GenericTileBoardState {
    self.board_state
  }
  fn board_state_mut(&mut self) -> &mut GenericTileBoardState {
    &mut self.board_state
  }

  fn line_state(&self, line: Line) -> GenericTileBoardState {
    self.line_states[line.idx()]
  }
  fn line_state_mut(&mut self, line: Line) -> &mut GenericTileBoardState {
    &mut self.line_states[line.idx()]
  }
}

/// Trait to allow recursion on inductive tile hierarchy.
trait TileTrait {
  fn tile_state(&self) -> GenericTileBoardState;

  fn trivial_tile(&self, pos_iter: impl Iterator<Item = GenericPos>) -> TrivialTile;
  fn place_symbol(&mut self, pos_iter: impl Iterator<Item = GenericPos>, symbol: PlayerSymbol);
}

/// Base case of the inductive tile hierarchy.
impl TileTrait for TrivialTile {
  fn tile_state(&self) -> GenericTileBoardState {
    (*self).into()
  }

  fn trivial_tile(&self, mut pos_iter: impl Iterator<Item = GenericPos>) -> TrivialTile {
    assert!(pos_iter.next().is_none());
    *self
  }
  fn place_symbol(&mut self, mut pos_iter: impl Iterator<Item = GenericPos>, symbol: PlayerSymbol) {
    assert!(pos_iter.next().is_none());
    *self = TrivialTile::new_occupied(symbol);
  }
}

/// Induction step of the inductive tile hierarchy.
impl<BoardType: BoardTrait> TileTrait for BoardType {
  fn tile_state(&self) -> GenericTileBoardState {
    self.board_state()
  }
  fn trivial_tile(&self, pos_iter: impl Iterator<Item = GenericPos>) -> TrivialTile {
    BoardTrait::trivial_tile(self, pos_iter)
  }
  fn place_symbol(&mut self, pos_iter: impl Iterator<Item = GenericPos>, symbol: PlayerSymbol) {
    BoardTrait::place_symbol(self, pos_iter, symbol)
  }
}

/// A `GenericTileBoardState` is a state inside the board hierarchy.
/// It can be seen as both a tile state and a board state,
/// depending on what level of the hierarchy you are considering.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GenericTileBoardState {
  #[default]
  FreeUndecided,
  OccupiedWon(PlayerSymbol),
  UnoccupiableDraw,
}

impl GenericTileBoardState {
  pub fn is_free(self) -> bool {
    matches!(self, Self::FreeUndecided)
  }
  pub fn is_occupied_won(self) -> bool {
    matches!(self, Self::OccupiedWon(_))
  }
  pub fn is_unoccupiable_draw(self) -> bool {
    matches!(self, Self::UnoccupiableDraw)
  }

  /// combinator used to compute the state of a whole line
  fn line_combinator(self, other: Self) -> Self {
    match [self, other] {
      [Self::OccupiedWon(s1), Self::OccupiedWon(s2)] if s1 == s2 => Self::OccupiedWon(s1),
      [Self::FreeUndecided, Self::FreeUndecided] => Self::FreeUndecided,
      [Self::OccupiedWon(_), Self::FreeUndecided] => Self::FreeUndecided,
      [Self::FreeUndecided, Self::OccupiedWon(_)] => Self::FreeUndecided,
      _ => Self::UnoccupiableDraw,
    }
  }
}

/// Trivial tile state at the bottom of the board hierarchy.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrivialTile(pub Option<PlayerSymbol>);

impl TrivialTile {
  pub fn new_free() -> Self {
    Self(None)
  }
  pub fn new_occupied(symbol: PlayerSymbol) -> Self {
    Self(Some(symbol))
  }
  pub fn is_free(self) -> bool {
    self.0.is_none()
  }
  pub fn is_occupied(self) -> bool {
    self.0.is_some()
  }
}

impl From<TrivialTile> for GenericTileBoardState {
  fn from(tile: TrivialTile) -> Self {
    match tile {
      TrivialTile(None) => Self::FreeUndecided,
      TrivialTile(Some(symbol)) => Self::OccupiedWon(symbol),
    }
  }
}
