use crate::LOCAL_BOARD_SIZE;

use super::pos::GenericPos;

const NLINES: usize = 2 * LOCAL_BOARD_SIZE as usize + 2;

/// guranteed to be valid
#[derive(Clone, Copy)]
pub(crate) enum Line {
  XAxis(u8),
  YAxis(u8),
  MainDiagonal,
  AntiDiagonal,
}
impl Line {
  pub(crate) fn x_axis(y: u8) -> Self {
    assert!(y < LOCAL_BOARD_SIZE);
    Self::XAxis(y)
  }
  pub(crate) fn y_axis(x: u8) -> Self {
    assert!(x < LOCAL_BOARD_SIZE);
    Self::YAxis(x)
  }

  pub(crate) fn idx(self) -> usize {
    let line_length = LOCAL_BOARD_SIZE as usize;
    match self {
      Line::XAxis(y) => {
        assert!(y < LOCAL_BOARD_SIZE);
        y as usize
      }
      Line::YAxis(x) => {
        assert!(x < LOCAL_BOARD_SIZE);
        x as usize + line_length
      }
      Line::MainDiagonal => 2 * line_length,
      Line::AntiDiagonal => 2 * line_length + 1,
    }
  }
  pub(crate) fn from_idx(idx: usize) -> Self {
    assert!(idx < NLINES);
    let idx = idx as u8;
    if (0..LOCAL_BOARD_SIZE).contains(&idx) {
      Line::XAxis(idx)
    } else if (LOCAL_BOARD_SIZE..2 * LOCAL_BOARD_SIZE).contains(&idx) {
      Line::YAxis(idx - LOCAL_BOARD_SIZE)
    } else if idx == 2 * LOCAL_BOARD_SIZE {
      Line::MainDiagonal
    } else {
      Line::AntiDiagonal
    }
  }
  pub(crate) fn iter(self) -> LineIter {
    LineIter {
      line_type: self,
      i: 0,
    }
  }

  pub(crate) fn all_through_point(pos: GenericPos) -> impl Iterator<Item = Self> {
    let mut l = vec![Self::x_axis(pos.y()), Self::y_axis(pos.x())];
    if pos.x() == pos.y() {
      l.push(Self::MainDiagonal);
    }
    if pos.x() + pos.y() == 2 {
      l.push(Self::AntiDiagonal);
    }
    l.into_iter()
  }

  pub(crate) fn all() -> impl Iterator<Item = Self> {
    (0..NLINES).map(Self::from_idx)
  }
}

pub(crate) struct LineIter {
  line_type: Line,
  i: u8,
}
impl Iterator for LineIter {
  type Item = GenericPos;

  fn next(&mut self) -> Option<Self::Item> {
    if self.i >= 3 {
      return None;
    }
    let i = self.i;
    self.i += 1;
    Some(match self.line_type {
      Line::XAxis(y) => GenericPos::new(i, y),
      Line::YAxis(x) => GenericPos::new(x, i),
      Line::MainDiagonal => GenericPos::new(i, i),
      Line::AntiDiagonal => GenericPos::new(i, 2 - i),
    })
  }
}
