use crate::{
  pos::{InnerPos, OuterPos},
  PlayerSymbol,
};

use std::{
  io::{Read, Write},
  net::TcpStream,
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::trace;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
  SymbolAssignment(PlayerSymbol),
  GameStart(PlayerSymbol),
  ChooseInnerBoardRejected,
  ChooseInnerBoardAccepted(OuterPos),
  PlaceSymbolRejected,
  PlaceSymbolAccepted(InnerPos),
}

impl ServerMessage {
  pub fn symbol_assigment(self) -> PlayerSymbol {
    match self {
      Self::SymbolAssignment(s) => s,
      _ => panic!("expected `SymbolAssignment`, got `{:?}`", self),
    }
  }
  pub fn game_start(self) -> PlayerSymbol {
    match self {
      Self::GameStart(s) => s,
      _ => panic!("wong message: expected `GameStart`, got `{:?}`", self),
    }
  }
  pub fn choose_inner_board_rejected(self) {
    match self {
      Self::ChooseInnerBoardRejected => (),
      _ => panic!("expected `ChooseInnerBoardRejected`, got `{:?}`", self),
    }
  }
  pub fn choose_inner_board_accepted(self) -> OuterPos {
    match self {
      Self::ChooseInnerBoardAccepted(p) => p,
      _ => panic!("expected `ChooseInnerBoardAccepted`, got `{:?}`", self),
    }
  }
  pub fn place_symbol_rejected(self) {
    match self {
      Self::PlaceSymbolRejected => (),
      _ => panic!("expected `PlaceSymbolRejected`, got `{:?}`", self),
    }
  }
  pub fn place_symbol_accepted(self) -> InnerPos {
    match self {
      Self::PlaceSymbolAccepted(p) => p,
      _ => panic!("expected `PlaceSymbolAccepted`, got `{:?}`", self),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
  ChooseInnerBoardProposal(OuterPos),
  PlaceSymbolProposal(InnerPos),
}

impl ClientMessage {
  pub fn choose_inner_board_proposal(self) -> OuterPos {
    match self {
      Self::ChooseInnerBoardProposal(p) => p,
      _ => panic!("expected `ChooseInnerBoardProposal`, got `{:?}`", self),
    }
  }
  pub fn place_symbol_proposal(self) -> InnerPos {
    match self {
      Self::PlaceSymbolProposal(p) => p,
      _ => panic!("expected `PlaceSymbolProposal`, got `{:?}`", self),
    }
  }
}

pub fn send_message_to_stream<Msg: Serialize + std::fmt::Debug>(
  msg: &Msg,
  stream: &mut TcpStream,
) -> eyre::Result<()> {
  let msg_string = ron::to_string(msg)?;
  let msg_len = msg_string.len() as u64;
  let msg_len_bytes = msg_len.to_be_bytes();
  let msg_bytes = msg_string.as_bytes();
  stream.write_all(&msg_len_bytes)?;
  stream.write_all(msg_bytes)?;
  trace!(
    "Sent message to {}\n{:?}",
    stream.local_addr().unwrap().ip(),
    msg
  );
  Ok(())
}

pub fn receive_message_from_stream<Msg: DeserializeOwned + std::fmt::Debug>(
  stream: &mut TcpStream,
) -> eyre::Result<Msg> {
  let mut msg_len_bytes = [0u8; 8];
  stream.read_exact(&mut msg_len_bytes)?;
  let msg_len = u64::from_be_bytes(msg_len_bytes) as usize;
  let mut msg_bytes = vec![0u8; msg_len];
  stream.read_exact(&mut msg_bytes)?;
  let msg_string = String::from_utf8(msg_bytes)?;
  let msg = ron::from_str(&msg_string)?;
  trace!(
    "Received message from {}\n{:?}",
    stream.local_addr().unwrap().ip(),
    msg
  );
  Ok(msg)
}
