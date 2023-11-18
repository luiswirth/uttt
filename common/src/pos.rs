use serde::{Deserialize, Serialize};

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
  pub const fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 3 && arr[1] < 3);
    Self(arr)
  }
  pub const fn new(x: u8, y: u8) -> Self {
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

  pub fn is_on_main_diagonal(self) -> bool {
    self.x() == self.y()
  }
  pub fn is_on_anti_diagonal(self) -> bool {
    self.x() + self.y() == 2
  }

  pub fn lines(self) -> impl Iterator<Item = LineIter> {
    let mut l = vec![LineIter::x_axis(self.y()), LineIter::y_axis(self.x())];
    if self.is_on_main_diagonal() {
      l.push(LineIter::main_diagonal());
    }
    if self.is_on_anti_diagonal() {
      l.push(LineIter::anti_diagonal());
    }
    l.into_iter()
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

enum LineType {
  XAxis(u8),
  YAxis(u8),
  MainDiagonal,
  AntiDiagonal,
}

pub struct LineIter {
  line_type: LineType,
  i: u8,
}
impl LineIter {
  pub fn x_axis(y: u8) -> Self {
    assert!(y < 3);
    Self {
      line_type: LineType::XAxis(y),
      i: 0,
    }
  }
  pub fn y_axis(x: u8) -> Self {
    assert!(x < 3);
    Self {
      line_type: LineType::YAxis(x),
      i: 0,
    }
  }
  pub fn main_diagonal() -> Self {
    Self {
      line_type: LineType::MainDiagonal,
      i: 0,
    }
  }
  pub fn anti_diagonal() -> Self {
    Self {
      line_type: LineType::AntiDiagonal,
      i: 0,
    }
  }
}
impl Iterator for LineIter {
  type Item = LocalPos;

  fn next(&mut self) -> Option<Self::Item> {
    if self.i >= 3 {
      return None;
    }
    let i = self.i;
    self.i += 1;
    Some(match self.line_type {
      LineType::XAxis(y) => LocalPos::new(i, y),
      LineType::YAxis(x) => LocalPos::new(x, i),
      LineType::MainDiagonal => LocalPos::new(i, i),
      LineType::AntiDiagonal => LocalPos::new(i, 2 - i),
    })
  }
}
