pub mod message;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

pub const PLAYER_SYMBOLS: [PlayerSymbol; 2] = [PlayerSymbol::Cross, PlayerSymbol::Circle];

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlayerSymbol {
  Cross = 0,
  Circle = 1,
}

impl Distribution<PlayerSymbol> for rand::distributions::Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PlayerSymbol {
    PlayerSymbol::from_idx(rng.gen_range(0..2))
  }
}
impl PlayerSymbol {
  pub fn to_idx(self) -> usize {
    self as usize
  }
  pub fn from_idx(idx: usize) -> Self {
    PLAYER_SYMBOLS[idx]
  }
  pub fn other(self) -> Self {
    use PlayerSymbol as P;
    match self {
      P::Cross => P::Circle,
      P::Circle => P::Cross,
    }
  }
  pub fn switch(&mut self) {
    *self = self.other();
  }
}

pub struct GameState {
  pub board: OuterBoard,
  pub current_player: PlayerSymbol,
  pub current_inner_board: Option<OuterPos>,
}
impl GameState {
  pub fn new(starting_player: PlayerSymbol) -> Self {
    Self {
      current_player: starting_player,
      board: Default::default(),
      current_inner_board: Default::default(),
    }
  }
  pub fn place_symbol(&mut self, inner_pos: InnerPos, symbol: PlayerSymbol) {
    self.board.set(
      (
        self
          .current_inner_board
          .expect("InnerBoard needs to be already chosen."),
        inner_pos,
      ),
      symbol,
    );
  }
}

#[derive(Default, Serialize, Deserialize)]
pub struct OuterBoard {
  pub inners: [InnerBoard; 9],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalPos([u8; 2]);
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OuterPos([u8; 2]);
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerPos([u8; 2]);

impl GlobalPos {
  pub fn linear_idx(self) -> usize {
    (self.0[0] * 9 + self.0[1]) as usize
  }
}
impl OuterPos {
  pub fn new(x: u8, y: u8) -> Self {
    Self([x, y])
  }
  pub fn linear_idx(self) -> usize {
    (self.0[0] * 3 + self.0[1]) as usize
  }
}
impl InnerPos {
  pub fn new(x: u8, y: u8) -> Self {
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

impl OuterBoard {
  pub fn get(&mut self, global_pos: GlobalPos) -> Option<PlayerSymbol> {
    let outer_pos = OuterPos::from(global_pos);
    let inner_pos = InnerPos::from(global_pos);
    self.inners[outer_pos.linear_idx()].tiles[inner_pos.linear_idx()].state
  }
  pub fn set(&mut self, global_pos: impl Into<GlobalPos>, symbol: PlayerSymbol) {
    let global_pos = global_pos.into();
    let outer_pos = OuterPos::from(global_pos);
    let inner_pos = InnerPos::from(global_pos);
    self.inners[outer_pos.linear_idx()].tiles[inner_pos.linear_idx()] = Tile::occupied(symbol);
  }
}

#[derive(Default, Serialize, Deserialize)]
pub struct InnerBoard {
  pub state: InnerBoardState,
  pub tiles: [Tile; 9],
}

#[derive(Default, Serialize, Deserialize)]
pub enum InnerBoardState {
  #[default]
  Free,
  Occupied(PlayerSymbol),
  Drawn,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Tile {
  pub state: Option<PlayerSymbol>,
}
impl Tile {
  pub fn occupied(symbol: PlayerSymbol) -> Self {
    Self {
      state: Some(symbol),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameEndType {
  Win(PlayerSymbol),
  Forfeit,
  Draw,
}

#[derive(Serialize, Deserialize)]
pub enum MessageActor {
  Client(PlayerSymbol),
  Server,
}
