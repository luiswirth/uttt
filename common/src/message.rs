use super::{GameEndType, GameState, PlayerSymbol};

use serde::Serialize;

#[derive(Serialize)]
pub struct Message {
  pub full_state: GameState,
  pub specific: MessageSpecific,
}

#[derive(Serialize)]
pub enum MessageSpecific {
  Client(ClientMessage),
  Server(ServerMessage),
}

#[derive(Serialize)]
pub enum ClientMessage {
  ConnectionRequest { username: String },
  GameRestartRequest,
  TryChooseInnerBoardRequest { pos: [u8; 2] },
  TryMoveRequest { pos: [u8; 2] },
  ForfeitMessage,
}

#[derive(Serialize)]
pub enum ServerMessage {
  ConnectionResponse {
    success: bool,
    username: String,
    symbol: PlayerSymbol,
  },
  GameStartMessage,
  ChooseInnerBoardMessage,
  TryChooseInnerBoardResponse {
    success: bool,
    pos: [u8; 2],
  },
  NewTurnMessage {
    player: PlayerSymbol,
    inner_board: [u8; 2],
  },

  TryMoveResponse {
    success: bool,
    pos: [u8; 2],
  },
  GameEndingMessage(GameEndType),
}
