use serde::{Deserialize, Serialize};
use tungstenite::Message;
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;

/// Internal message type for ipc
#[derive(Serialize, Deserialize, Clone)]
pub(super) enum __Message {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<__CloseFrame>),
}

impl From<Message> for __Message {
    fn from(value: Message) -> Self {
        match value {
            Message::Text(text) => Self::Text(text.to_string()),
            Message::Binary(payload) => Self::Binary(payload.to_vec()),
            Message::Ping(payload) => Self::Ping(payload.to_vec()),
            Message::Pong(payload) => Self::Pong(payload.to_vec()),
            Message::Close(frame) => Self::Close(frame.map(Into::into)),
            Message::Frame(_) => unreachable!(),
        }
    }
}

impl From<__Message> for Message {
    fn from(value: __Message) -> Self {
        match value {
            __Message::Text(text) => Self::text(text),
            __Message::Binary(payload) => Self::binary(payload),
            __Message::Ping(payload) => Self::Ping(payload.into()),
            __Message::Pong(payload) => Self::Pong(payload.into()),
            __Message::Close(frame) => Self::Close(frame.map(Into::into)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct __CloseFrame {
    code: __CloseCode,
    reason: String,
}

impl From<CloseFrame> for __CloseFrame {
    fn from(value: CloseFrame) -> Self {
        Self {
            code: value.code.into(),
            reason: value.reason.to_string(),
        }
    }
}

impl From<__CloseFrame> for CloseFrame {
    fn from(value: __CloseFrame) -> Self {
        Self {
            code: value.code.into(),
            reason: value.reason.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub(super) struct __CloseCode(u16);

impl From<CloseCode> for __CloseCode {
    fn from(code: CloseCode) -> Self {
        __CloseCode(code.into())
    }
}

impl From<__CloseCode> for CloseCode {
    fn from(code: __CloseCode) -> Self {
        code.0.into()
    }
}
