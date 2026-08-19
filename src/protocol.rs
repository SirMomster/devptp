use serde::{Deserialize, Serialize};

pub const ALPN: &[u8] = b"devptp/0";
pub const STREAM_NAME: &str = "devptp_ipc";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Request {
    Ping,
    StartServing,
    ExposePort { local_port: u16 },
    Connect { ticket: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Response {
    Pong,
    Status { running: bool },
    Ok,
    Error { message: String },
    ServingStarted { ticket: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Message {
    Ping,
    Pong,
    Text { value: String },

    // Daemon -> peer:
    // "Start listening here; connections should reach my local_port."
    ExposeTcp { local_port: u16 },

    // Peer -> daemon, over a newly opened bi stream:
    // "This stream represents a connection to this daemon-side port."
    ForwardTcp { port: u16 },
    UnexposeTcp { local_port: u16 },
}
