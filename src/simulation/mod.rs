use std::borrow::Cow;

use serde_json::Value;
use socketioxide::{BroadcastError, SocketIo};

pub mod implementation;
pub mod runner;

pub enum ThreadEvent {
    Shutdown,
    ChangeTargetTPS(usize),
}

pub fn ws_emit(
    socket_io: SocketIo,
    event: impl Into<Cow<'static, str>> + std::marker::Send + 'static,
    data: Value,
) -> Result<(), BroadcastError> {
    socket_io.of("/simulation").unwrap().emit(event, data)
}
