use crate::{PlayerSymbol, LOCAL_BOARD_SIZE};

use super::{board::TileBoardState, pos::Pos};

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

  pub(crate) fn all_through_point(pos: Pos) -> impl Iterator<Item = Self> {
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
  type Item = Pos;

  fn next(&mut self) -> Option<Self::Item> {
    if self.i >= 3 {
      return None;
    }
    let i = self.i;
    self.i += 1;
    Some(match self.line_type {
      Line::XAxis(y) => Pos::new(i, y),
      Line::YAxis(x) => Pos::new(x, i),
      Line::MainDiagonal => Pos::new(i, i),
      Line::AntiDiagonal => Pos::new(i, 2 - i),
    })
  }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum LineState {
  #[default]
  Free,
  PartiallyWon(PlayerSymbol, u8),
  Won(PlayerSymbol),
  Drawn,
}

impl LineState {
  pub fn is_drawn(self) -> bool {
    matches!(self, Self::Drawn)
  }

  /// combinator used to compute the state of a whole line
  pub fn combinator(self, other: Self) -> Self {
    match [self, other] {
      [Self::PartiallyWon(p0, n0), Self::PartiallyWon(p1, n1)] if p0 == p1 => {
        let n = n0 + n1;
        debug_assert!(n <= LOCAL_BOARD_SIZE);
        if n == LOCAL_BOARD_SIZE {
          Self::Won(p0)
        } else {
          Self::PartiallyWon(p0, n)
        }
      }
      [s @ Self::PartiallyWon(..), Self::Free] => s,
      [Self::Free, s @ Self::PartiallyWon(..)] => s,
      [Self::Free, Self::Free] => Self::Free,
      [Self::Won(_), _] | [_, Self::Won(_)] => panic!("a winning line should never be combined"),
      _ => Self::Drawn,
    }
  }
}

impl From<TileBoardState> for LineState {
  fn from(t: TileBoardState) -> Self {
    match t {
      TileBoardState::Free => Self::Free,
      TileBoardState::Won(p) => Self::PartiallyWon(p, 1),
      TileBoardState::Drawn => Self::Drawn,
    }
  }
}
