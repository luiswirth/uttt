use crate::{GlobalPos, PlayerSymbol};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
  io::{Read, Write},
  net::TcpStream,
};

type MessageLength = u32;
const NBYTES_MESSAGE_LENGTH: usize = std::mem::size_of::<MessageLength>();

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServerMsgSymbolAssignment(pub PlayerSymbol);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServerMsgRoundStart(pub PlayerSymbol);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClientReqRoundStart;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MsgPlayerAction {
  MakeMove(GlobalPos),
  GiveUp,
}

impl MsgPlayerAction {
  pub fn make_move(self) -> GlobalPos {
    match self {
      Self::MakeMove(p) => p,
      _ => panic!("unexpected player action: {:?}", self),
    }
  }
  pub fn opponent_give_up(self) {
    match self {
      Self::GiveUp => (),
      _ => panic!("unexpected player action: {:?}", self),
    }
  }
}

/// Sends a message (any serializable type) to the given stream.
/// This function uses `write_all`, so it will block until the full message is sent.
/// Therefore it is not suitable for `no_blocking` streams.
pub fn send_msg_to_stream<Msg: Serialize>(
  msg: &Msg,
  stream: &mut TcpStream,
) -> std::io::Result<()> {
  let msg_string = ron::to_string(msg).expect("serialization failed");
  let msg_bytes = msg_string.as_bytes();
  let msg_len = MessageLength::try_from(msg_bytes.len()).expect("message too long");
  let msg_len_bytes = msg_len.to_be_bytes();
  stream.write_all(&msg_len_bytes)?;
  stream.write_all(msg_bytes)?;
  Ok(())
}
/// Receives a message (any serializable type) from the given stream.
/// This function uses `read_exact`, so it will block until the full message is received.
/// Therefore it is not suitable for `no_blocking` streams.
pub fn receive_msg_from_stream<Msg: DeserializeOwned>(
  stream: &mut TcpStream,
) -> std::io::Result<Msg> {
  let mut msg_len_bytes = [0u8; NBYTES_MESSAGE_LENGTH];
  stream.read_exact(&mut msg_len_bytes)?;
  let msg_len = MessageLength::from_be_bytes(msg_len_bytes);
  let msg_len_usize = usize::try_from(msg_len).expect("message too long");
  let mut msg_bytes = vec![0u8; msg_len_usize];
  stream.read_exact(&mut msg_bytes)?;
  let msg_string = String::from_utf8(msg_bytes).expect("message not valid UTF-8");
  let msg = ron::from_str(&msg_string).expect("deserialization failed");
  Ok(msg)
}

const SINGLE_READ_BUFFER_SIZE: usize = 1024;

/// A datastructure for handling partial reads from a stream.
/// Suitable for a non-blocking stream.
pub struct MessageIoHandlerNoBlocking {
  stream: TcpStream,
  raw_read_data: Vec<u8>,
  raw_write_data: Vec<u8>,
}
impl MessageIoHandlerNoBlocking {
  pub fn new(stream: TcpStream) -> Self {
    Self {
      stream,
      raw_read_data: Vec::new(),
      raw_write_data: Vec::new(),
    }
  }

  /// Does partial reads from the stream until to receive messages.
  /// Returns `Ok(Some(msg))` if a full message was built, `Err(e)` if an error occurred.
  /// If the message is not complete, returns `Ok(None)`.
  pub fn try_read_msg<Msg: DeserializeOwned>(&mut self) -> std::io::Result<Option<Msg>> {
    // do a single read
    let mut read_buffer = [0u8; SINGLE_READ_BUFFER_SIZE];
    let nbytes_read = match self.stream.read(&mut read_buffer) {
      Ok(n) => n,
      Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => 0,
      Err(e) => return Err(e),
    };
    let read_bytes = &read_buffer[..nbytes_read];
    self.raw_read_data.extend_from_slice(read_bytes);

    // try to get message length
    if self.raw_read_data.len() < NBYTES_MESSAGE_LENGTH {
      return Ok(None);
    }
    let msg_len_bytes = &self.raw_read_data[..NBYTES_MESSAGE_LENGTH];
    let msg_len_bytes: [u8; NBYTES_MESSAGE_LENGTH] = msg_len_bytes.try_into().unwrap();
    let msg_len = MessageLength::from_be_bytes(msg_len_bytes);
    let msg_len_usize = usize::try_from(msg_len).expect("message too long");
    let whole_msg_len = NBYTES_MESSAGE_LENGTH + msg_len_usize;

    // try to get message content
    if self.raw_read_data.len() < whole_msg_len {
      return Ok(None);
    }
    let msg_content_bytes = &self.raw_read_data[NBYTES_MESSAGE_LENGTH..whole_msg_len];
    let msg_str = std::str::from_utf8(msg_content_bytes).expect("not valid UTF-8");
    let msg = ron::from_str(msg_str).expect("deserialization failed");

    self.raw_read_data.drain(..whole_msg_len);
    Ok(Some(msg))
  }

  /// Does partial writes to the stream until all messages are sent.
  /// Returns `Ok(true)` if all messages were sent, `Ok(false)` if there are still messages to send, `Err(e)` if an error occurred.
  pub fn try_write_msg<Msg: Serialize>(&mut self, msg: Option<Msg>) -> std::io::Result<bool> {
    if let Some(msg) = msg {
      let msg_string = ron::to_string(&msg).expect("serialization failed");
      let msg_bytes = msg_string.as_bytes();
      let msg_len = MessageLength::try_from(msg_bytes.len()).expect("message too long");
      let msg_len_bytes = msg_len.to_be_bytes();
      self.raw_write_data.extend_from_slice(&msg_len_bytes);
      self.raw_write_data.extend_from_slice(msg_bytes);
    }

    let nbytes_written = match self.stream.write(&self.raw_write_data) {
      Ok(n) => n,
      Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => 0,
      Err(e) => return Err(e),
    };

    self.raw_write_data.drain(..nbytes_written);

    Ok(self.raw_write_data.is_empty())
  }
}
