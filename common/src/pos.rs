use serde::{Deserialize, Serialize};

/// instance guranteed to be valid
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OuterPos([u8; 2]);

impl OuterPos {
  pub fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 3 && arr[1] < 3);
    Self(arr)
  }
  pub fn new(x: u8, y: u8) -> Self {
    Self::new_arr([x, y])
  }

  pub fn x(self) -> u8 {
    self.0[0]
  }
  pub fn y(self) -> u8 {
    self.0[1]
  }
  pub fn linear_idx(self) -> usize {
    (self.x() * 3 + self.y()) as usize
  }
}

/// instance guranteed to be valid
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerPos([u8; 2]);

impl InnerPos {
  pub fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 3 && arr[1] < 3);
    Self(arr)
  }
  pub fn new(x: u8, y: u8) -> Self {
    Self::new_arr([x, y])
  }

  pub fn x(self) -> u8 {
    self.0[0]
  }
  pub fn y(self) -> u8 {
    self.0[1]
  }
  pub fn linear_idx(self) -> usize {
    (self.x() * 3 + self.y()) as usize
  }

  pub fn as_outer(self) -> OuterPos {
    OuterPos(self.0)
  }
}

/// represents either a outer pos or a inner pos
/// instance guranteed to be valid
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LocalPos([u8; 2]);

impl LocalPos {
  pub fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 3 && arr[1] < 3);
    Self(arr)
  }
  pub fn new(x: u8, y: u8) -> Self {
    Self::new_arr([x, y])
  }

  pub fn x(self) -> u8 {
    self.0[0]
  }
  pub fn y(self) -> u8 {
    self.0[1]
  }
  pub fn linear_idx(self) -> usize {
    (self.x() * 3 + self.y()) as usize
  }

  pub fn as_outer(self) -> OuterPos {
    OuterPos(self.0)
  }
  pub fn as_inner(self) -> InnerPos {
    InnerPos(self.0)
  }

  pub fn is_main_diagonal(self) -> bool {
    self.x() == self.y()
  }
  pub fn is_anti_diagonal(self) -> bool {
    self.x() + self.y() == 2
  }

  // TODO: consider moving these methods outside of this struct

  pub fn x_axis(self) -> impl Iterator<Item = Self> {
    (0..3).map(move |y| Self::new(self.x(), y))
  }
  pub fn y_axis(self) -> impl Iterator<Item = Self> {
    (0..3).map(move |x| Self::new(x, self.y()))
  }
  pub fn main_diagonal(self) -> Option<impl Iterator<Item = Self>> {
    self
      .is_main_diagonal()
      .then(|| (0..3).map(move |i| Self::new(i, i)))
  }
  pub fn anti_diagonal(self) -> Option<impl Iterator<Item = Self>> {
    self
      .is_anti_diagonal()
      .then(|| (0..3).map(move |i| Self::new(i, 2 - i)))
  }
}

impl From<OuterPos> for LocalPos {
  fn from(outer: OuterPos) -> Self {
    Self(outer.0)
  }
}
impl From<InnerPos> for LocalPos {
  fn from(inner: InnerPos) -> Self {
    Self(inner.0)
  }
}

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

  pub fn x(self) -> u8 {
    self.0[0]
  }
  pub fn y(self) -> u8 {
    self.0[1]
  }
  pub fn linear_idx(self) -> usize {
    (self.x() * 9 + self.y()) as usize
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
