use super::specific::{InnerPos, OuterPos};

/// represents either a outer pos or a inner pos
/// instance guranteed to be valid
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct GenericPos([u8; 2]);

impl GenericPos {
  pub(crate) const fn new_arr(arr: [u8; 2]) -> Self {
    assert!(arr[0] < 3 && arr[1] < 3);
    Self(arr)
  }
  pub(crate) const fn new(x: u8, y: u8) -> Self {
    Self::new_arr([x, y])
  }

  pub(crate) fn x(self) -> u8 {
    self.0[0]
  }
  pub(crate) fn y(self) -> u8 {
    self.0[1]
  }
  pub(crate) fn linear_idx(self) -> usize {
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
