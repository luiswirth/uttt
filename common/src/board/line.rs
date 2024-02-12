use super::{tile::TilePos, TileBoardState, BOARD_SIDE_LENGTH};
use crate::PlayerSymbol;

const NLINES: usize = 2 * BOARD_SIDE_LENGTH as usize + 2;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct LineState {
  occupant: Option<PlayerSymbol>,
  noccupied: u8,
}

#[allow(dead_code)]
impl LineState {
  pub fn new(occupant: Option<PlayerSymbol>, noccupied: u8) -> Self {
    assert!(noccupied <= BOARD_SIDE_LENGTH);
    assert!(noccupied != 0 || occupant.is_none());
    Self {
      occupant,
      noccupied,
    }
  }

  pub fn free() -> Self {
    Self::new(None, 0)
  }
  pub fn partially_won(player: PlayerSymbol, count: u8) -> Self {
    assert!(count != 0);
    Self::new(Some(player), count)
  }
  pub fn won(player: PlayerSymbol) -> Self {
    Self::new(Some(player), BOARD_SIDE_LENGTH)
  }
  pub fn drawn(count: u8) -> Self {
    assert!(count != 0);
    Self::new(None, count)
  }
  pub fn fully_drawn() -> Self {
    Self::new(None, BOARD_SIDE_LENGTH)
  }

  pub fn is_free(self) -> bool {
    if self.noccupied == 0 {
      debug_assert!(self.occupant.is_none());
      true
    } else {
      false
    }
  }

  pub fn is_partially_won(self) -> bool {
    self.occupant.is_none() && self.noccupied != 0
  }
  pub fn is_won(self) -> bool {
    self.occupant.is_some() && self.noccupied == BOARD_SIDE_LENGTH
  }
  pub fn is_drawn(self) -> bool {
    self.occupant.is_none() && self.noccupied != 0
  }
  pub fn is_fully_drawn(self) -> bool {
    self.occupant.is_none() && self.noccupied == BOARD_SIDE_LENGTH
  }

  pub fn winner(self) -> Option<PlayerSymbol> {
    match self.is_won() {
      true => self.occupant,
      false => None,
    }
  }

  /// combinator used to compute the state of a whole line
  pub fn combine(self, other: Self) -> Self {
    let noccupied = self.noccupied + other.noccupied;
    let occupant = match [self.occupant, other.occupant] {
      [a, b] if a == b => a,
      [_, b] if self.noccupied == 0 => b,
      [a, _] if other.noccupied == 0 => a,
      _ => None,
    };
    Self {
      occupant,
      noccupied,
    }
  }
}

impl From<TileBoardState> for LineState {
  fn from(t: TileBoardState) -> Self {
    match t {
      TileBoardState::Free => Self::free(),
      TileBoardState::Won(p) => Self::partially_won(p, 1),
      TileBoardState::Drawn | TileBoardState::FullyDrawn => Self::drawn(1),
    }
  }
}

/// A type of line in the board.
///
/// guranteed to be valid
#[derive(Clone, Copy)]
pub(crate) enum LinePos {
  XAxis(u8),
  YAxis(u8),
  MainDiagonal,
  AntiDiagonal,
}
impl LinePos {
  pub(crate) fn x_axis(y: u8) -> Self {
    assert!(y < BOARD_SIDE_LENGTH);
    Self::XAxis(y)
  }
  pub(crate) fn y_axis(x: u8) -> Self {
    assert!(x < BOARD_SIDE_LENGTH);
    Self::YAxis(x)
  }

  pub(crate) fn idx(self) -> usize {
    let line_length = BOARD_SIDE_LENGTH as usize;
    match self {
      LinePos::XAxis(y) => {
        assert!(y < BOARD_SIDE_LENGTH);
        y as usize
      }
      LinePos::YAxis(x) => {
        assert!(x < BOARD_SIDE_LENGTH);
        x as usize + line_length
      }
      LinePos::MainDiagonal => 2 * line_length,
      LinePos::AntiDiagonal => 2 * line_length + 1,
    }
  }
  pub(crate) fn from_idx(idx: usize) -> Self {
    assert!(idx < NLINES);
    let idx = idx as u8;
    if (0..BOARD_SIDE_LENGTH).contains(&idx) {
      LinePos::XAxis(idx)
    } else if (BOARD_SIDE_LENGTH..2 * BOARD_SIDE_LENGTH).contains(&idx) {
      LinePos::YAxis(idx - BOARD_SIDE_LENGTH)
    } else if idx == 2 * BOARD_SIDE_LENGTH {
      LinePos::MainDiagonal
    } else {
      LinePos::AntiDiagonal
    }
  }
  pub(crate) fn iter(self) -> LineTilePosIter {
    LineTilePosIter {
      line_pos: self,
      i: 0,
    }
  }

  pub(crate) fn all_through_point(pos: TilePos) -> impl Iterator<Item = Self> {
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

/// Iterator yielding `TilePos` along a line.
pub(crate) struct LineTilePosIter {
  line_pos: LinePos,
  i: u8,
}
impl Iterator for LineTilePosIter {
  type Item = TilePos;

  fn next(&mut self) -> Option<Self::Item> {
    if self.i >= 3 {
      return None;
    }
    let i = self.i;
    self.i += 1;
    Some(match self.line_pos {
      LinePos::XAxis(y) => TilePos::new(i, y),
      LinePos::YAxis(x) => TilePos::new(x, i),
      LinePos::MainDiagonal => TilePos::new(i, i),
      LinePos::AntiDiagonal => TilePos::new(i, 2 - i),
    })
  }
}

/// A container of tile states for a board.
/// Allows for easy indexing using TilePos.
#[derive(Debug, Default)]
pub struct LineStates([LineState; 8]);
impl std::ops::Index<LinePos> for LineStates {
  type Output = LineState;
  fn index(&self, line: LinePos) -> &Self::Output {
    &self.0[line.idx()]
  }
}
impl std::ops::IndexMut<LinePos> for LineStates {
  fn index_mut(&mut self, line: LinePos) -> &mut Self::Output {
    &mut self.0[line.idx()]
  }
}

#[cfg(test)]
mod test {
  use super::LineState;
  use crate::{board::BOARD_SIDE_LENGTH, PLAYERS};

  #[test]
  fn check_line_state_cominator() {
    use LineState as L;

    assert_eq!(L::free().combine(L::free()), L::free());

    for p in PLAYERS {
      let o = p.other();
      assert_eq!(
        L::free().combine(L::partially_won(p, 1)),
        L::partially_won(p, 1)
      );
      assert_eq!(
        L::partially_won(p, 1).combine(L::partially_won(p, 1)),
        L::partially_won(p, 2)
      );
      assert_eq!(
        L::partially_won(p, 1).combine(L::partially_won(o, 1)),
        L::drawn(2)
      );
      assert_eq!(
        L::partially_won(p, BOARD_SIDE_LENGTH - 1).combine(L::partially_won(p, 1)),
        L::won(p)
      );
      assert_eq!(
        L::drawn(BOARD_SIDE_LENGTH - 1).combine(L::drawn(1)),
        L::fully_drawn()
      );
      assert_eq!(
        L::drawn(BOARD_SIDE_LENGTH - 1).combine(L::partially_won(p, 1)),
        L::fully_drawn()
      );
    }
  }
}
