use std::{io, sync::Arc};

use interprocess::local_socket::{
    GenericFilePath, ListenerOptions, ToFsName, tokio::Stream, traits::tokio::Listener,
};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::{io::BufReader, sync::oneshot};

use crate::{
    daemon::Daemon,
    error::Result,
    helpers::{receive_json, send_json},
    protocol::{Request, Response, ipc_socket_path},
};

pub struct IPCManager {
    pub daemon: Arc<Daemon>,
}

#[derive(Deserialize)]
struct PortParams {
    local_port: u16,
}

#[derive(Deserialize)]
struct TicketParams {
    ticket: String,
}

impl IPCManager {
    pub fn new(daemon: Arc<Daemon>) -> Self {
        Self { daemon }
    }

    pub async fn start_ipc_receiver(
        self: Arc<Self>,
        ready: oneshot::Sender<io::Result<()>>,
    ) -> Result<()> {
        let path = ipc_socket_path();
        let name = path.clone().to_fs_name::<GenericFilePath>()?;
        let listener = match ListenerOptions::new().name(name).create_tokio() {
            Ok(listener) => listener,
            Err(error) => {
                let _ = ready.send(Err(io::Error::new(error.kind(), error.to_string())));
                return Err(error.into());
            }
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(error) =
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            {
                let _ = ready.send(Err(io::Error::new(error.kind(), error.to_string())));
                return Err(error.into());
            }
        }
        let _ = ready.send(Ok(()));
        println!("IPC server listening");

        loop {
            tokio::select! {
                _ = self.daemon.shutdown.notified() => break,
                result = listener.accept() => {
                    let stream = result?;
                    let manager = self.clone();
                    tokio::spawn(async move {
                        if let Err(error) = manager.handle_ipc_client(stream).await {
                            eprintln!("Client error: {error}");
                        }
                    });
                }
            }
        }
        Ok(())
    }

    async fn handle_ipc_client(self: Arc<Self>, stream: Stream) -> io::Result<()> {
        let (read_half, mut write_half) = tokio::io::split(stream);
        let mut reader = BufReader::new(read_half);

        loop {
            let request = match receive_json::<_, Request>(&mut reader).await {
                Ok(Some(request)) => request,
                Ok(None) => break,
                Err(error) => {
                    send_json(
                        &mut write_half,
                        &Response::failure(None, "invalid_request", error.to_string()),
                    )
                    .await?;
                    continue;
                }
            };

            let response = self.dispatch(request).await;
            let should_shutdown = response
                .result
                .as_ref()
                .and_then(|value| value.get("shutdown"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            send_json(&mut write_half, &response).await?;
            if should_shutdown {
                self.daemon.shutdown().await;
                break;
            }
        }
        Ok(())
    }

    async fn dispatch(&self, request: Request) -> Response {
        let id = request.id.clone();
        let result = match request.method.as_str() {
            "ping" => {
                let peer = self.daemon.client_peer.lock().await.clone();
                match peer {
                    Some(peer) => peer
                        .send(&crate::protocol::Message::Ping)
                        .await
                        .map(|_| json!({"pong": true})),
                    None => Err(crate::error::Error::Custom("No peer is connected".into())),
                }
            }
            "start_serving" => match self.daemon.start_receiver().await {
                Ok(()) => self
                    .daemon
                    .get_ticket()
                    .await
                    .map(|ticket| json!({"ticket": ticket})),
                Err(error) => Err(error),
            },
            "connect" => match self.parse_ticket(&request.params) {
                Ok(ticket) => self
                    .daemon
                    .start_peer(ticket)
                    .await
                    .map(|_| json!({"connected": true})),
                Err(error) => Err(error),
            },
            "expose_port" => match self.parse_port(&request.params) {
                Ok(port) => self
                    .daemon
                    .forward_local_service(port)
                    .await
                    .map(|_| json!({"local_port": port})),
                Err(error) => Err(error),
            },
            "status" => Ok(json!(self.daemon.status().await)),
            "list_forwarded_ports" => {
                Ok(json!({"ports": self.daemon.status().await.forwarded_ports}))
            }
            "disconnect" => {
                self.daemon.disconnect().await;
                Ok(json!({"connected": false}))
            }
            "shutdown" => Ok(json!({"shutdown": true})),
            _ => Err(crate::error::Error::Custom(format!(
                "Unknown method: {}",
                request.method
            ))),
        };

        match result {
            Ok(value) => Response::success(id, value),
            Err(error) => Response::failure(id, "command_failed", error.to_string()),
        }
    }

    fn parse_port(&self, params: &Value) -> Result<u16> {
        serde_json::from_value::<PortParams>(params.clone())
            .map(|params| params.local_port)
            .map_err(|error| crate::error::Error::Custom(error.to_string()))
    }

    fn parse_ticket(&self, params: &Value) -> Result<String> {
        serde_json::from_value::<TicketParams>(params.clone())
            .map(|params| params.ticket)
            .map_err(|error| crate::error::Error::Custom(error.to_string()))
    }
}
