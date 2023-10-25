pub mod message;

use serde::Serialize;

#[derive(Serialize)]
pub enum PlayerSymbol {
  Cross,
  Circle,
}

#[derive(Serialize)]
pub struct GameState {
  pub board: OuterBoard,
  pub current_player: PlayerSymbol,
}

#[derive(Default, Serialize)]
pub struct OuterBoard {
  pub inners: [InnerBoard; 9],
}

#[derive(Default, Serialize)]
pub struct InnerBoard {
  pub state: InnerBoardState,
  pub tiles: [Tile; 9],
}

#[derive(Default, Serialize)]
pub enum InnerBoardState {
  #[default]
  Free,
  Occupied(PlayerSymbol),
  Drawn,
}

#[derive(Default, Serialize)]
pub struct Tile {
  pub state: Option<PlayerSymbol>,
}

#[derive(Serialize)]
pub enum GameEndType {
  Win(PlayerSymbol),
  Forfeit,
  Draw,
}

#[derive(Serialize)]
pub enum MessageActor {
  Client(PlayerSymbol),
  Server,
}
