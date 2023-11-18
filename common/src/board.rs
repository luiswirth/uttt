use crate::{
  pos::{GlobalPos, InnerPos, OuterPos},
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
  pub fn tile(&self, global_pos: impl Into<GlobalPos>) -> &Tile {
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
  }
}

impl std::fmt::Display for OuterBoard {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for outer_y in 0..3 {
      for inner_y in 0..3 {
        for outer_x in 0..3 {
          for inner_x in 0..3 {
            let global_pos = GlobalPos::new(outer_x * 3 + inner_x, outer_y * 3 + inner_y);
            let symbol = self.tile(global_pos).state;
            let c = match symbol {
              Some(PlayerSymbol::Cross) => 'X',
              Some(PlayerSymbol::Circle) => 'O',
              None => '*',
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
  pub tiles: [Tile; 9],
}

impl InnerBoard {
  pub fn tile(&self, pos: InnerPos) -> &Tile {
    &self.tiles[pos.linear_idx()]
  }
  pub fn tile_mut(&mut self, pos: InnerPos) -> &mut Tile {
    &mut self.tiles[pos.linear_idx()]
  }

  pub fn place_symbol(&mut self, pos: InnerPos, symbol: PlayerSymbol) {
    assert!(self.state.is_free());
    let tile = self.tile_mut(pos);
    assert!(tile.is_free());
    *tile = Tile::new_occupied(symbol);
  }

  pub fn is_free(&self) -> bool {
    self.state.is_free()
  }
}

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
pub struct Tile {
  pub state: Option<PlayerSymbol>,
}
impl Tile {
  pub fn new_occupied(symbol: PlayerSymbol) -> Self {
    Self {
      state: Some(symbol),
    }
  }
  pub fn is_free(self) -> bool {
    self.state.is_none()
  }
  pub fn is_occupied(self) -> bool {
    self.state.is_some()
  }
}
