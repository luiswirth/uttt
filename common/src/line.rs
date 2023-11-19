use crate::{pos::LocalPos, LOCAL_BOARD_SIZE};

const NLINES: usize = 2 * LOCAL_BOARD_SIZE as usize + 2;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineType {
  XAxis(u8),
  YAxis(u8),
  MainDiagonal,
  AntiDiagonal,
}
impl LineType {
  pub fn x_axis(y: u8) -> Self {
    assert!(y < LOCAL_BOARD_SIZE);
    Self::XAxis(y)
  }
  pub fn y_axis(x: u8) -> Self {
    assert!(x < LOCAL_BOARD_SIZE);
    Self::YAxis(x)
  }

  pub fn idx(self) -> usize {
    let line_length = LOCAL_BOARD_SIZE as usize;
    match self {
      LineType::XAxis(y) => {
        assert!(y < LOCAL_BOARD_SIZE);
        y as usize
      }
      LineType::YAxis(x) => {
        assert!(x < LOCAL_BOARD_SIZE);
        x as usize + line_length
      }
      LineType::MainDiagonal => 2 * line_length,
      LineType::AntiDiagonal => 2 * line_length + 1,
    }
  }
  pub fn from_idx(idx: usize) -> Self {
    assert!(idx < NLINES);
    let idx = idx as u8;
    if (0..LOCAL_BOARD_SIZE).contains(&idx) {
      LineType::XAxis(idx)
    } else if (LOCAL_BOARD_SIZE..2 * LOCAL_BOARD_SIZE).contains(&idx) {
      LineType::YAxis(idx - LOCAL_BOARD_SIZE)
    } else if idx == 2 * LOCAL_BOARD_SIZE {
      LineType::MainDiagonal
    } else {
      LineType::AntiDiagonal
    }
  }
  pub fn iter(self) -> LineIter {
    LineIter {
      line_type: self,
      i: 0,
    }
  }

  pub fn all_through_point(pos: LocalPos) -> impl Iterator<Item = Self> {
    let mut l = vec![Self::x_axis(pos.y()), Self::y_axis(pos.x())];
    if pos.x() == pos.y() {
      l.push(Self::MainDiagonal);
    }
    if pos.x() + pos.y() == 2 {
      l.push(Self::AntiDiagonal);
    }
    l.into_iter()
  }

  pub fn all() -> impl Iterator<Item = Self> {
    (0..NLINES).map(Self::from_idx)
  }
}

pub struct LineIter {
  line_type: LineType,
  i: u8,
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
