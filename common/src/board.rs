pub mod line;
pub mod tile;

use line::{LinePos, LineState, LineStates};
use tile::{TilePos, TileStates, TileTrait, TrivialTileState};

use crate::PlayerSymbol;

pub const BOARD_SIDE_LENGTH: u8 = 3;
pub const BOARD_AREA: u8 = BOARD_SIDE_LENGTH * BOARD_SIDE_LENGTH;

/// `TrivialBoard` is the bottom of the board hierarchy.
/// It is the base case of the inductive type `GenericBoard`.
pub type TrivialBoard = GenericBoard<TrivialTileState>;

/// Inductive board type generating the board hierarchy.
/// The generic [`TileType`] only needs to implement the [`TileTrait`].
#[allow(private_bounds)]
#[derive(Debug, Default)]
pub struct GenericBoard<TileType: TileTrait> {
  /// ground tile states
  tile_states: TileStates<TileType>,

  /// derived line states (redundant information)
  line_states: LineStates,
  /// derived board state (redundant information)
  board_state: TileBoardState,
}

#[allow(private_bounds)]
impl<TileType: TileTrait> GenericBoard<TileType> {
  pub fn tile_state(&self, pos: impl Into<TilePos>) -> &TileType {
    &self.tile_states[pos.into()]
  }
  pub fn line_state(&self, pos: impl Into<LinePos>) -> LineState {
    self.line_states[pos.into()]
  }
  pub fn board_state(&self) -> TileBoardState {
    self.board_state
  }

  /// Returns the trivial tile at the given position, by recursively walking the board hierarchy.
  pub fn trivial_tile(&self, pos_iter: impl IntoIterator<Item = TilePos>) -> TrivialTileState {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");
    self.tile_states[local_pos].trivial_tile_in_tile(pos_iter)
  }

  pub fn could_place_symbol(&self, pos_iter: impl IntoIterator<Item = TilePos>) -> bool {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");

    self.board_state().is_placeable()
      && self.tile_states[local_pos].could_place_symbol_in_tile(pos_iter)
  }

  /// Tries to place a symbol on the given trivial tile, by recursively walking the board hierarchy and
  /// updating the state of the `TrivialTile` and the hierarchy of super states.
  pub fn try_place_symbol(
    &mut self,
    pos_iter: impl IntoIterator<Item = TilePos>,
    symbol: PlayerSymbol,
  ) -> Result<(), PlaceSymbolError> {
    let mut pos_iter = pos_iter.into_iter();
    let local_pos = pos_iter.next().expect("ran out of positions");

    if self.board_state().is_placeable() {
      self.tile_states[local_pos].try_place_symbol_in_tile(pos_iter, symbol)?;
      self.update_super_states(local_pos);
      Ok(())
    } else {
      Err(PlaceSymbolError::BoardNotPlaceable)
    }
  }

  /// Updates the local super states (line and board states), after a tile at the given pos has changed.
  fn update_super_states(&mut self, local_pos: TilePos) {
    for line in LinePos::all_through_point(local_pos) {
      let line_state = line
        .iter()
        .map(|pos| LineState::from(self.tile_states[pos].tile_state()))
        .reduce(|a, b| a.combine(b))
        .unwrap();

      self.line_states[line] = line_state;
      if let Some(p) = line_state.winner() {
        self.board_state = TileBoardState::Won(p);
      }
    }
    if LinePos::all().all(|line| self.line_states[line].is_drawn()) {
      self.board_state = TileBoardState::Drawn;
    }
    if LinePos::all().all(|line| self.line_states[line].is_fully_drawn()) {
      self.board_state = TileBoardState::FullyDrawn;
    }
  }
}

/// A `TileBoardState` is a state inside the board hierarchy.
/// It can be seen as both a tile state and a board state,
/// depending on what level of the hierarchy you are considering.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TileBoardState {
  #[default]
  Free,
  Won(PlayerSymbol),
  Drawn,
  FullyDrawn,
}

impl TileBoardState {
  pub fn is_free(self) -> bool {
    matches!(self, Self::Free)
  }
  pub fn is_won(self) -> bool {
    matches!(self, Self::Won(_))
  }
  pub fn is_drawn(self) -> bool {
    matches!(self, Self::Drawn | Self::FullyDrawn)
  }
  pub fn is_fully_drawn(self) -> bool {
    matches!(self, Self::FullyDrawn)
  }

  pub fn is_decided(self) -> bool {
    self.is_won() || self.is_drawn()
  }
  pub fn is_placeable(self) -> bool {
    !self.is_won() && !self.is_fully_drawn()
  }
}

impl From<TrivialTileState> for TileBoardState {
  fn from(tile: TrivialTileState) -> Self {
    match tile {
      TrivialTileState::Free => Self::Free,
      TrivialTileState::Won(player) => Self::Won(player),
    }
  }
}

#[derive(Debug)]
pub enum PlaceSymbolError {
  BoardNotPlaceable,
  TrivialTileNotFree,
}

#[cfg(test)]
mod test {
  use std::str::FromStr;

  use super::{
    tile::{TilePos, TrivialTileState},
    TileBoardState, TrivialBoard, BOARD_AREA,
  };

  #[derive(Debug)]
  pub enum TrivialBoardParseError {
    InvalidChar(char),
    BadSymbolCount,
  }

  impl FromStr for TrivialBoard {
    type Err = TrivialBoardParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
      let mut board = TrivialBoard::default();

      let tiles: Result<Vec<_>, _> = s
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| TrivialTileState::from_char(c).ok_or(TrivialBoardParseError::InvalidChar(c)))
        .collect();
      let tiles: [TrivialTileState; BOARD_AREA as usize] = tiles?
        .try_into()
        .map_err(|_| TrivialBoardParseError::BadSymbolCount)?;

      for (i, tile) in tiles.into_iter().enumerate() {
        if let TrivialTileState::Won(p) = tile {
          let pos = TilePos::from_linear_idx(i);
          // TODO: maybe handle better
          board.try_place_symbol(pos.iter(), p).unwrap();
        }
      }

      Ok(board)
    }
  }

  #[test]
  fn check_draw_detection() {
    let board = r#"
      XOO
      OXX
      _XO
      "#
    .parse::<TrivialBoard>()
    .unwrap();
    assert_eq!(board.board_state, TileBoardState::Drawn);
  }
}
