pub mod message;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

pub const PLAYER_SYMBOLS: [PlayerSymbol; 2] = [PlayerSymbol::Cross, PlayerSymbol::Circle];

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlayerSymbol {
  Cross = 0,
  Circle = 1,
}

impl PlayerSymbol {
  pub fn to_idx(self) -> usize {
    self as usize
  }
  pub fn from_idx(idx: usize) -> Self {
    PLAYER_SYMBOLS[idx]
  }
  pub fn other(self) -> Self {
    match self {
      Self::Cross => Self::Circle,
      Self::Circle => Self::Cross,
    }
  }
  pub fn switch(&mut self) {
    *self = self.other();
  }
}

impl Distribution<PlayerSymbol> for rand::distributions::Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PlayerSymbol {
    PlayerSymbol::from_idx(rng.gen_range(0..2))
  }
}

#[derive(Default, Serialize, Deserialize)]
pub struct OuterBoard {
  pub inners: [InnerBoard; 9],
}

impl OuterBoard {
  pub fn get_inner_board(&self, outer_pos: OuterPos) -> &InnerBoard {
    &self.inners[outer_pos.linear_idx()]
  }
  pub fn set_inner_board(&mut self, global_pos: impl Into<GlobalPos>, symbol: PlayerSymbol) {
    let global_pos = global_pos.into();
    let outer_pos = OuterPos::from(global_pos);
    let inner_pos = InnerPos::from(global_pos);
    self.inners[outer_pos.linear_idx()].place_symbol(inner_pos, symbol);
  }

  pub fn tile(&self, global_pos: impl Into<GlobalPos>) -> Option<PlayerSymbol> {
    let global_pos = global_pos.into();
    let outer_pos = OuterPos::from(global_pos);
    let inner_pos = InnerPos::from(global_pos);
    self.inners[outer_pos.linear_idx()].tile(inner_pos)
  }
  pub fn place_symbol(&mut self, global_pos: impl Into<GlobalPos>, symbol: PlayerSymbol) {
    let global_pos = global_pos.into();
    let outer_pos = OuterPos::from(global_pos);
    let inner_pos = InnerPos::from(global_pos);
    self.inners[outer_pos.linear_idx()].place_symbol(inner_pos, symbol);
  }
}

#[derive(Default, Serialize, Deserialize)]
pub struct InnerBoard {
  pub state: InnerBoardState,
  pub tiles: [Tile; 9],
}

impl InnerBoard {
  pub fn tile(&self, inner_pos: InnerPos) -> Option<PlayerSymbol> {
    self.tiles[inner_pos.linear_idx()].state
  }
  pub fn place_symbol(&mut self, inner_pos: InnerPos, symbol: PlayerSymbol) {
    self.tiles[inner_pos.linear_idx()] = Tile::new_occupied(symbol);
  }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Tile {
  pub state: Option<PlayerSymbol>,
}
impl Tile {
  pub fn new_occupied(symbol: PlayerSymbol) -> Self {
    Self {
      state: Some(symbol),
    }
  }

  pub fn is_occupied(self) -> bool {
    self.state.is_some()
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalPos(pub [u8; 2]);
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OuterPos(pub [u8; 2]);
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerPos(pub [u8; 2]);

impl GlobalPos {
  pub fn linear_idx(self) -> usize {
    (self.0[0] * 9 + self.0[1]) as usize
  }
}
impl OuterPos {
  pub fn new(x: u8, y: u8) -> Self {
    assert!(x < 3 && y < 3);
    Self([x, y])
  }
  pub fn linear_idx(self) -> usize {
    (self.0[0] * 3 + self.0[1]) as usize
  }
}
impl InnerPos {
  pub fn new(x: u8, y: u8) -> Self {
    assert!(x < 3 && y < 3);
    Self([x, y])
  }
  pub fn linear_idx(self) -> usize {
    (self.0[0] * 3 + self.0[1]) as usize
  }
  pub fn as_outer(self) -> OuterPos {
    OuterPos(self.0)
  }
}

impl From<(OuterPos, InnerPos)> for GlobalPos {
  fn from((outer, inner): (OuterPos, InnerPos)) -> Self {
    Self([outer.0[0] * 3 + inner.0[0], outer.0[1] * 3 + inner.0[1]])
  }
}
impl From<GlobalPos> for OuterPos {
  fn from(global: GlobalPos) -> Self {
    Self(global.0.map(|v| v / 3))
  }
}
impl From<GlobalPos> for InnerPos {
  fn from(global: GlobalPos) -> Self {
    Self(global.0.map(|v| v - (v / 3) * 3))
  }
}