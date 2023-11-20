use super::specific::{InnerPos, OuterPos};

/// Generic local positions inside board hierarchy.
///
/// Instance guranteed to be valid.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GenericPos([u8; 2]);

impl GenericPos {
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
}

impl From<OuterPos> for GenericPos {
  fn from(outer: OuterPos) -> Self {
    Self(outer.0)
  }
}
impl From<InnerPos> for GenericPos {
  fn from(inner: InnerPos) -> Self {
    Self(inner.0)
  }
}
