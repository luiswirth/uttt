use crate::generic::board::{GenericBoard, TrivialBoard};

/// `OuterBoard` is the first non-trivial board in the board hierarchy.
pub type OuterBoard = GenericBoard<TrivialBoard>;
