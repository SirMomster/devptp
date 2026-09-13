use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

pub const ALPN: &[u8] = b"devptp/0";
pub const STREAM_NAME: &str = "devptp_ipc";

pub fn ipc_socket_path() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("devptp.sock")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: Option<Value>,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: String,
    pub message: String,
}

impl Response {
    pub fn success(id: Option<Value>, result: impl Serialize) -> Self {
        Self {
            id,
            ok: true,
            result: Some(serde_json::to_value(result).unwrap_or(Value::Null)),
            error: None,
        }
    }

    pub fn failure(id: Option<Value>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id,
            ok: false,
            result: None,
            error: Some(ResponseError {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Message {
    Ping,
    Pong,
    Text { value: String },
    NewKnownPorts { value: Vec<u16> },
    NewRemovedPorts { value: Vec<u16> },
    KnownPorts { value: Vec<u16> },
    GetPorts,
    ExposeTcp { local_port: u16 },
    ForwardTcp { port: u16 },
    UnexposeTcp { local_port: u16 },
}
