use super::{GenericBoard, PlaceSymbolError, TileBoardState};

use crate::PlayerSymbol;

/// Trait to allow recursion on inductive tile hierarchy.
pub(crate) trait TileTrait {
  fn tile_state(&self) -> TileBoardState;
  fn trivial_tile_in_tile(&self, pos_iter: impl Iterator<Item = TilePos>) -> TrivialTileState;

  fn could_place_symbol_in_tile(&self, pos_iter: impl Iterator<Item = TilePos>) -> bool;
  fn try_place_symbol_in_tile(
    &mut self,
    pos_iter: impl Iterator<Item = TilePos>,
    symbol: PlayerSymbol,
  ) -> Result<(), PlaceSymbolError>;
}

/// Induction step of the inductive tile hierarchy.
impl<TileType: TileTrait> TileTrait for GenericBoard<TileType> {
  fn tile_state(&self) -> TileBoardState {
    self.board_state()
  }
  fn trivial_tile_in_tile(&self, pos_iter: impl Iterator<Item = TilePos>) -> TrivialTileState {
    GenericBoard::trivial_tile(self, pos_iter)
  }

  fn could_place_symbol_in_tile(&self, pos_iter: impl Iterator<Item = TilePos>) -> bool {
    GenericBoard::could_place_symbol(self, pos_iter)
  }
  fn try_place_symbol_in_tile(
    &mut self,
    pos_iter: impl Iterator<Item = TilePos>,
    symbol: PlayerSymbol,
  ) -> Result<(), PlaceSymbolError> {
    GenericBoard::try_place_symbol(self, pos_iter, symbol)
  }
}

/// The trivial tile is at the bottom of the tile hierarchy.
#[derive(Debug, Clone, Copy, Default)]
pub enum TrivialTileState {
  #[default]
  Free,
  Won(PlayerSymbol),
}

/// Base case of the inductive tile hierarchy.
impl TileTrait for TrivialTileState {
  fn tile_state(&self) -> TileBoardState {
    (*self).into()
  }

  fn trivial_tile_in_tile(&self, mut pos_iter: impl Iterator<Item = TilePos>) -> TrivialTileState {
    assert!(pos_iter.next().is_none());
    *self
  }

  fn could_place_symbol_in_tile(&self, mut pos_iter: impl Iterator<Item = TilePos>) -> bool {
    assert!(pos_iter.next().is_none());
    self.is_free()
  }
  fn try_place_symbol_in_tile(
    &mut self,
    mut pos_iter: impl Iterator<Item = TilePos>,
    symbol: PlayerSymbol,
  ) -> Result<(), PlaceSymbolError> {
    assert!(pos_iter.next().is_none());
    if self.is_free() {
      *self = TrivialTileState::Won(symbol);
      Ok(())
    } else {
      Err(PlaceSymbolError::TrivialTileNotFree)
    }
  }
}

impl TrivialTileState {
  pub fn is_free(self) -> bool {
    matches!(self, Self::Free)
  }
  pub fn is_won(self) -> bool {
    matches!(self, Self::Won(_))
  }

  pub fn as_char(self) -> char {
    match self {
      Self::Free => '_',
      Self::Won(p) => p.as_char(),
    }
  }
  pub fn from_char(c: char) -> Option<Self> {
    match c {
      '_' => Some(Self::Free),
      _ => PlayerSymbol::from_char(c).map(Self::Won),
    }
  }
}

/// A specific tile position in a board.
/// This position is purely local and only describes the tiles
/// location in relation to it's direct board.
///
/// Instance guranteed to be valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TilePos([u8; 2]);

impl TilePos {
  pub const fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 3 && arr[1] < 3);
    Self(arr)
  }
  pub const fn new(x: u8, y: u8) -> Self {
    Self::new_arr([x, y])
  }

  pub fn x(self) -> u8 {
    self.0[0]
  }
  pub fn y(self) -> u8 {
    self.0[1]
  }
  pub fn linear_idx(self) -> usize {
    (self.x() * 3 + self.y()) as usize
  }
  pub fn from_linear_idx(idx: usize) -> Self {
    Self::new(idx as u8 / 3, idx as u8 % 3)
  }

  pub fn iter(self) -> impl Iterator<Item = Self> {
    std::iter::once(self)
  }
}

/// A container of tile states for a board.
/// Allows for easy indexing using TilePos.
#[derive(Debug, Default)]
pub struct TileStates<T>([T; 9]);
impl<T> std::ops::Index<TilePos> for TileStates<T> {
  type Output = T;
  fn index(&self, pos: TilePos) -> &Self::Output {
    &self.0[pos.linear_idx()]
  }
}
impl<T> std::ops::IndexMut<TilePos> for TileStates<T> {
  fn index_mut(&mut self, pos: TilePos) -> &mut Self::Output {
    &mut self.0[pos.linear_idx()]
  }
}
