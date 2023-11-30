pub mod generic;
pub mod specific;

use std::net::Ipv4Addr;

use rand::prelude::*;
use serde::{Deserialize, Serialize};

pub const DEFAULT_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
pub const DEFAULT_PORT: u16 = 42069;

pub const NPLAYERS: u8 = 2;
pub const LOCAL_BOARD_SIZE: u8 = 3;
pub const PLAYERS: [Player; 2] = [Player::Cross, Player::Circle];

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Player {
  Cross = 0,
  Circle = 1,
}

impl Player {
  pub fn idx(self) -> usize {
    self as usize
  }
  pub fn from_idx(idx: usize) -> Self {
    PLAYERS[idx]
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

impl Distribution<Player> for rand::distributions::Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Player {
    Player::from_idx(rng.gen_range(0..2))
  }
}
