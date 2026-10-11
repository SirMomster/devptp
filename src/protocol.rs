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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_round_trips_with_default_params() {
        let request: Request = serde_json::from_value(json!({
            "id": 7,
            "method": "status"
        }))
        .expect("request should deserialize");

        assert_eq!(request.id, Some(json!(7)));
        assert_eq!(request.method, "status");
        assert_eq!(request.params, Value::Null);
    }

    #[test]
    fn responses_preserve_success_and_failure_shapes() {
        let success = serde_json::to_value(Response::success(
            Some(json!(1)),
            json!({
                "pong": true
            }),
        ))
        .expect("success should serialize");
        assert_eq!(
            success,
            json!({
                "id": 1,
                "ok": true,
                "result": {"pong": true}
            })
        );

        let failure = serde_json::to_value(Response::failure(
            Some(json!(2)),
            "command_failed",
            "No peer is connected",
        ))
        .expect("failure should serialize");
        assert_eq!(
            failure,
            json!({
                "id": 2,
                "ok": false,
                "error": {
                    "code": "command_failed",
                    "message": "No peer is connected"
                }
            })
        );
    }

    #[test]
    fn message_wire_names_remain_stable() {
        assert_eq!(
            serde_json::to_value(Message::ExposeTcp { local_port: 8080 }).unwrap(),
            json!({"type": "expose_tcp", "data": {"local_port": 8080}})
        );
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
