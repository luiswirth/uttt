use tracing::info;

use crate::{
  pos::{GlobalPos, InnerPos, LocalPos, OuterPos},
  PlayerSymbol,
};

#[derive(Default)]
pub struct OuterBoard {
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

    // TODO: check winning condition in outterboard
    let inner_board_state = self.inners[outer_pos.linear_idx()].place_symbol(inner_pos, symbol);
    if let InnerBoardState::Occupied(winner) = inner_board_state {
      info!("InnerBoard {:?} won by {:?}.", outer_pos, winner);
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
            let c = match self.inner_board(global_pos.into()).state {
              InnerBoardState::Free => match self.tile(global_pos).0 {
                Some(sym) => sym.char(),
                None => '*',
              },
              InnerBoardState::Occupied(sym) => sym.char(),
              InnerBoardState::Drawn => '#',
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
  pub state: InnerBoardState,
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
  pub fn place_symbol(&mut self, pos: InnerPos, symbol: PlayerSymbol) -> InnerBoardState {
    assert!(self.state.is_free());
    let tile = self.tile_mut(pos);
    assert!(tile.is_free());
    *tile = TileState::new_occupied(symbol);
    self.update_state(pos)
  }

  pub fn is_free(&self) -> bool {
    self.state.is_free()
  }

  // called by place_symbol
  fn update_state(&mut self, new_tile_pos: InnerPos) -> InnerBoardState {
    if let TileState(Some(winner)) = LocalPos::from(new_tile_pos)
      .x_axis()
      .map(|pos| *self.tile(pos.as_inner()))
      .reduce(|a, b| a.merge_same(b))
      .unwrap()
    {
      self.state = InnerBoardState::Occupied(winner);
      return self.state;
    }

    if let TileState(Some(winner)) = LocalPos::from(new_tile_pos)
      .y_axis()
      .map(|pos| *self.tile(pos.as_inner()))
      .reduce(|a, b| a.merge_same(b))
      .unwrap()
    {
      self.state = InnerBoardState::Occupied(winner);
      return self.state;
    }

    if let Some(main_diagonal) = LocalPos::from(new_tile_pos).main_diagonal() {
      if let TileState(Some(winner)) = main_diagonal
        .map(|pos| *self.tile(pos.as_inner()))
        .reduce(|a, b| a.merge_same(b))
        .unwrap()
      {
        self.state = InnerBoardState::Occupied(winner);
        return self.state;
      }
    }
    if let Some(anti_diagonal) = LocalPos::from(new_tile_pos).anti_diagonal() {
      if let TileState(Some(winner)) = anti_diagonal
        .map(|pos| *self.tile(pos.as_inner()))
        .reduce(|a, b| a.merge_same(b))
        .unwrap()
      {
        self.state = InnerBoardState::Occupied(winner);
        return self.state;
      }
    }
    self.state
  }
}

// TODO: consider replacing `Free` by wrapping this as `Option<InnerBoadState>`
#[derive(Debug, Clone, Copy, Default)]
pub enum InnerBoardState {
  #[default]
  Free,
  Occupied(PlayerSymbol),
  Drawn,
}

impl InnerBoardState {
  pub fn is_free(self) -> bool {
    matches!(self, Self::Free)
  }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TileState(pub Option<PlayerSymbol>);

impl TileState {
  pub fn new_occupied(symbol: PlayerSymbol) -> Self {
    Self(Some(symbol))
  }
  pub fn is_free(self) -> bool {
    self.0.is_none()
  }
  pub fn is_occupied(self) -> bool {
    self.0.is_some()
  }
  pub fn merge_same(self, other: Self) -> Self {
    match (self.0, other.0) {
      (Some(a), Some(b)) if a == b => Self(Some(a)),
      _ => Self(None),
    }
  }
}
