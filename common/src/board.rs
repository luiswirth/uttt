use crate::{
  pos::{GlobalPos, InnerPos, LineIter, LocalPos, OuterPos},
  PlayerSymbol,
};

#[derive(Default)]
pub struct OuterBoard {
  pub state: MetaTileBoardState,
  pub inners: [InnerBoard; 9],
}

impl OuterBoard {
  pub fn inner_board(&self, outer_pos: OuterPos) -> &InnerBoard {
    &self.inners[outer_pos.linear_idx()]
  }
  pub fn tile(&self, global_pos: impl Into<GlobalPos>) -> &TileState {
    let global_pos = global_pos.into();
    let outer_pos = OuterPos::from(global_pos);
    let inner_pos = InnerPos::from(global_pos);
    self.inners[outer_pos.linear_idx()].tile(inner_pos)
  }
  pub fn place_symbol(&mut self, pos: impl Into<GlobalPos>, symbol: PlayerSymbol) {
    let global_pos = pos.into();
    let outer_pos = OuterPos::from(global_pos);
    let inner_pos = InnerPos::from(global_pos);

    self.inners[outer_pos.linear_idx()].place_symbol(inner_pos, symbol);
    self.state = self.compute_super_state(outer_pos);
  }
}

impl std::fmt::Display for OuterBoard {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for outer_y in 0..3 {
      for inner_y in 0..3 {
        for outer_x in 0..3 {
          for inner_x in 0..3 {
            let global_pos = GlobalPos::new(outer_x * 3 + inner_x, outer_y * 3 + inner_y);
            let c = match self.inner_board(global_pos.into()).state {
              MetaTileBoardState::FreeUndecided => self.tile(global_pos).char(),
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

#[derive(Default)]
pub struct InnerBoard {
  pub state: MetaTileBoardState,
  pub tiles: [TileState; 9],
}

impl InnerBoard {
  pub fn tile(&self, pos: InnerPos) -> &TileState {
    &self.tiles[pos.linear_idx()]
  }
  pub fn tile_mut(&mut self, pos: InnerPos) -> &mut TileState {
    &mut self.tiles[pos.linear_idx()]
  }

  // returns (potentially new) state of the inner board
  pub fn place_symbol(&mut self, pos: InnerPos, symbol: PlayerSymbol) {
    assert!(self.state.is_free());
    let tile = self.tile_mut(pos);
    assert!(tile.is_free());
    *tile = TileState::new_occupied(symbol);
    self.state = self.compute_super_state(pos);
  }

  pub fn is_free(&self) -> bool {
    self.state.is_free()
  }
}

/// A `MetaTileBoardState` is a state inside the tile/board hierarchy.
/// It can be seen as both a tile state and a board state,
/// depending on what level of the hierarchy you are considering.
#[derive(Debug, Clone, Copy, Default)]
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
  pub fn merge_line(self, other: Self) -> Self {
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
pub struct TileState(pub Option<PlayerSymbol>);

impl TileState {
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

/// this is both a board and a tile
trait MetaTileBoard {
  /// viewed as board: position type of a tile
  type ConcreteLocalPos: Into<LocalPos>;

  /// <Self as Tile>: occupancy state of the tile
  /// <Self as Board>: winning state of the board
  fn sub_state(&self, pos: LocalPos) -> MetaTileBoardState;

  fn compute_line_state(&self, line: LineIter) -> MetaTileBoardState {
    line
      .map(|pos| self.sub_state(pos))
      .reduce(|a, b| a.merge_line(b))
      .unwrap()
  }

  /// computes the super state, based on the change of a sub state
  /// <Self as Board>: compute winning state of the board
  fn compute_super_state(&self, new_sub_state_pos: Self::ConcreteLocalPos) -> MetaTileBoardState {
    let new_tile_pos = new_sub_state_pos.into();

    for line in new_tile_pos.lines() {
      let line_state = self.compute_line_state(line);
      if line_state.is_occupied_won() {
        return line_state;
      }
    }

    MetaTileBoardState::FreeUndecided
  }
}

impl MetaTileBoard for InnerBoard {
  type ConcreteLocalPos = InnerPos;

  /// the sub state here is the trivial tile state
  fn sub_state(&self, pos: LocalPos) -> MetaTileBoardState {
    match self.tile(pos.as_inner()) {
      TileState(Some(s)) => MetaTileBoardState::OccupiedWon(*s),
      TileState(None) => MetaTileBoardState::FreeUndecided,
    }
  }
}

impl MetaTileBoard for OuterBoard {
  type ConcreteLocalPos = OuterPos;
  fn sub_state(&self, pos: LocalPos) -> MetaTileBoardState {
    self.inner_board(pos.as_outer()).state
  }
}
