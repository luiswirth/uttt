pub mod generic;
pub mod specific;

pub use generic::*;
pub use specific::*;

use crate::PlayerSymbol;

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
