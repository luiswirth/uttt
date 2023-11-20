use serde::{Deserialize, Serialize};

use super::GenericPos;

/// instance guranteed to be valid
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
  type Item = GenericPos;
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
  type Item = GenericPos;
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OuterPos(pub(super) [u8; 2]);

impl OuterPos {
  pub fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 3 && arr[1] < 3);
    Self(arr)
  }
  pub fn new(x: u8, y: u8) -> Self {
    Self::new_arr([x, y])
  }
}

/// instance guranteed to be valid
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerPos(pub(super) [u8; 2]);

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
