pub mod board;
pub mod game;
pub mod message;

use std::net::{Ipv4Addr, SocketAddrV4};

use board::{pos::Pos, GenericBoard, TrivialBoard};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

pub const DEFAULT_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
pub const DEFAULT_PORT: u16 = 42069;
pub const DEFAULT_SOCKET_ADDR: SocketAddrV4 = SocketAddrV4::new(DEFAULT_IP, DEFAULT_PORT);

pub const NPLAYERS: u8 = 2;
pub const PLAYERS: [PlayerSymbol; NPLAYERS as usize] = [PlayerSymbol::X, PlayerSymbol::O];

/// `OuterBoard` is the first non-trivial board in the board hierarchy.
pub type OuterBoard = GenericBoard<InnerBoard>;
pub type InnerBoard = TrivialBoard;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlayerSymbol {
  X = 0,
  O = 1,
}

impl PlayerSymbol {
  pub fn idx(self) -> usize {
    self as usize
  }
  pub fn from_idx(idx: usize) -> Self {
    PLAYERS[idx]
  }

  pub fn other(self) -> Self {
    match self {
      Self::X => Self::O,
      Self::O => Self::X,
    }
  }
  pub fn switch(&mut self) {
    *self = self.other();
  }

  pub fn as_char(self) -> char {
    match self {
      Self::X => 'X',
      Self::O => 'O',
    }
  }

  pub fn from_char(c: char) -> Option<Self> {
    match c {
      'X' => Some(Self::X),
      'O' => Some(Self::O),
      _ => None,
    }
  }
}

impl Distribution<PlayerSymbol> for rand::distributions::Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PlayerSymbol {
    PlayerSymbol::from_idx(rng.gen_range(0..2))
  }
}

/// instance guranteed to be valid
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalPos([u8; 2]);

impl GlobalPos {
  pub fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 9 && arr[1] < 9);
    Self(arr)
  }
  pub fn new(x: u8, y: u8) -> Self {
    Self::new_arr([x, y])
  }
}

impl IntoIterator for GlobalPos {
  type Item = Pos;
  type IntoIter = GlobalPosIter;
  fn into_iter(self) -> Self::IntoIter {
    GlobalPosIter { global: self, i: 0 }
  }
}

pub struct GlobalPosIter {
  global: GlobalPos,
  i: u8,
}
impl Iterator for GlobalPosIter {
  type Item = Pos;
  fn next(&mut self) -> Option<Self::Item> {
    match self.i {
      0 => {
        self.i = 1;
        Some(OuterPos::from(self.global).into())
      }
      1 => {
        self.i = 2;
        Some(InnerPos::from(self.global).into())
      }
      _ => None,
    }
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

/// instance guranteed to be valid
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OuterPos([u8; 2]);

impl OuterPos {
  pub fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 3 && arr[1] < 3);
    Self(arr)
  }
  pub fn new(x: u8, y: u8) -> Self {
    Self::new_arr([x, y])
  }
}

impl From<OuterPos> for Pos {
  fn from(outer: OuterPos) -> Self {
    Self::new_arr(outer.0)
  }
}

/// instance guranteed to be valid
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InnerPos([u8; 2]);

impl InnerPos {
  pub fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 3 && arr[1] < 3);
    Self(arr)
  }
  pub fn new(x: u8, y: u8) -> Self {
    Self::new_arr([x, y])
  }
  pub fn as_outer(self) -> OuterPos {
    OuterPos(self.0)
  }
}

impl From<InnerPos> for Pos {
  fn from(inner: InnerPos) -> Self {
    Self::new_arr(inner.0)
  }
}
