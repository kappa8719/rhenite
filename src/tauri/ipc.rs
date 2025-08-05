use serde::{Deserialize, Serialize};

#[cfg(feature = "tauri-host")]
type TungsteniteMessage = tungstenite::Message;
#[cfg(feature = "tauri-host")]
type TungsteniteCloseFrame = tungstenite::protocol::CloseFrame;
#[cfg(feature = "tauri-host")]
type TungsteniteCloseCode = tungstenite::protocol::frame::coding::CloseCode;

/// Internal message type for ipc
#[derive(Serialize, Deserialize, Clone)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}

#[cfg(feature = "tauri-host")]
impl From<TungsteniteMessage> for Message {
    fn from(value: TungsteniteMessage) -> Self {
        match value {
            TungsteniteMessage::Text(text) => Self::Text(text.to_string()),
            TungsteniteMessage::Binary(payload) => Self::Binary(payload.to_vec()),
            TungsteniteMessage::Ping(payload) => Self::Ping(payload.to_vec()),
            TungsteniteMessage::Pong(payload) => Self::Pong(payload.to_vec()),
            TungsteniteMessage::Close(frame) => Self::Close(frame.map(Into::into)),
            TungsteniteMessage::Frame(_) => unreachable!(),
        }
    }
}

#[cfg(feature = "tauri-host")]
impl From<Message> for TungsteniteMessage {
    fn from(value: Message) -> Self {
        match value {
            Message::Text(text) => Self::text(text),
            Message::Binary(payload) => Self::binary(payload),
            Message::Ping(payload) => Self::Ping(payload.into()),
            Message::Pong(payload) => Self::Pong(payload.into()),
            Message::Close(frame) => Self::Close(frame.map(Into::into)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CloseFrame {
    code: CloseCode,
    reason: String,
}

#[cfg(feature = "tauri-host")]
impl From<TungsteniteCloseFrame> for CloseFrame {
    fn from(value: TungsteniteCloseFrame) -> Self {
        Self {
            code: value.code.into(),
            reason: value.reason.to_string(),
        }
    }
}

#[cfg(feature = "tauri-host")]
impl From<CloseFrame> for TungsteniteCloseFrame {
    fn from(value: CloseFrame) -> Self {
        Self {
            code: value.code.into(),
            reason: value.reason.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct CloseCode(u16);

#[cfg(feature = "tauri-host")]
impl From<TungsteniteCloseCode> for CloseCode {
    fn from(code: TungsteniteCloseCode) -> Self {
        CloseCode(code.into())
    }
}

#[cfg(feature = "tauri-host")]
impl From<CloseCode> for TungsteniteCloseCode {
    fn from(code: CloseCode) -> Self {
        code.0.into()
    }
}
