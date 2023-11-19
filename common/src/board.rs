use crate::{
  line::LineType,
  pos::{GlobalPos, InnerPos, LocalPos, OuterPos},
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
    assert!(self.board_state.is_free());
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
    assert!(self.board_state.is_free());
    let tile = self.tile_mut(pos.into());
    assert!(tile.is_free());
    *tile = TrivialTileState::new_occupied(symbol);
    self.update_super_states(pos.into());
  }

  pub fn is_free(&self) -> bool {
    self.board_state.is_free()
  }
}
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GenericBoard<TileType: TileTrait> {
  tiles: [TileType; 9],
  line_states: [MetaTileBoardState; 8],
  board_state: MetaTileBoardState,
}

impl<TileType: TileTrait> GenericBoard<TileType> {
  pub fn board_state(&self) -> MetaTileBoardState {
    self.board_state
  }
}

// These methods are private, because they are shouldn't be used outside of this module.
// The "tile" terminology is also confusing outside of this module.
// The public interface is implemented specifically on the OuterBoard and InnerBoard types.
impl<TileType: TileTrait> GenericBoard<TileType> {
  fn tile(&self, pos: LocalPos) -> &TileType {
    &self.tiles[pos.linear_idx()]
  }
  fn tile_mut(&mut self, pos: LocalPos) -> &mut TileType {
    &mut self.tiles[pos.linear_idx()]
  }

  /// updates lines and board state after tile is changed
  fn update_super_states(&mut self, changed_tile_pos: LocalPos) {
    for line in LineType::all_through_point(changed_tile_pos) {
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
    if LineType::all().all(|line| self.line_states[line.idx()].is_unoccupiable_draw()) {
      self.board_state = MetaTileBoardState::UnoccupiableDraw;
    }
  }
}

pub trait TileTrait {
  fn state(&self) -> MetaTileBoardState;
}

impl TileTrait for TrivialTileState {
  fn state(&self) -> MetaTileBoardState {
    match self.0 {
      Some(s) => MetaTileBoardState::OccupiedWon(s),
      None => MetaTileBoardState::FreeUndecided,
    }
  }
}

impl<InnerTile: TileTrait> TileTrait for GenericBoard<InnerTile> {
  fn state(&self) -> MetaTileBoardState {
    self.board_state
  }
}

/// A `MetaTileBoardState` is a state inside the tile/board hierarchy.
/// It can be seen as both a tile state and a board state,
/// depending on what level of the hierarchy you are considering.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MetaTileBoardState {
  #[default]
  FreeUndecided,
  OccupiedWon(PlayerSymbol),
  UnoccupiableDraw,
}

impl MetaTileBoardState {
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

/// trivial tile state at the bottom of the tile/board hierarchy
#[derive(Debug, Clone, Copy, Default)]
pub struct TrivialTileState(pub Option<PlayerSymbol>);

impl TrivialTileState {
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

  pub fn char(self) -> char {
    match self.0 {
      Some(s) => s.char(),
      None => '*',
    }
  }
}

impl std::fmt::Display for OuterBoard {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for outer_y in 0..3 {
      for inner_y in 0..3 {
        for outer_x in 0..3 {
          for inner_x in 0..3 {
            let global_pos = GlobalPos::new(outer_x * 3 + inner_x, outer_y * 3 + inner_y);
            let outer_pos = OuterPos::from(global_pos);
            let inner_pos = InnerPos::from(global_pos);
            let inner_board = self.tile(outer_pos.into());
            let c = match inner_board.board_state {
              MetaTileBoardState::FreeUndecided => inner_board.tile(inner_pos.into()).char(),
              MetaTileBoardState::OccupiedWon(sym) => sym.char(),
              MetaTileBoardState::UnoccupiableDraw => '#',
            };
            write!(f, "{}", c)?;
          }
          write!(f, " ")?;
        }
        writeln!(f)?;
      }
      writeln!(f)?;
    }
    Ok(())
  }
}
