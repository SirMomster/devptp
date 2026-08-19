use iroh::protocol::ProtocolHandler;
use std::sync::Arc;
use tokio::sync::{mpsc::Receiver, Mutex};

use super::Response;
use super::super::error::Result;

use iroh::{endpoint::Connection, protocol::AcceptError};

pub const COMMAND_ALPN: &[u8] = b"devptp/commands/0";

#[derive(Debug, Clone)]
pub enum InnerProtocol {
    NewPort { port: u16 },
}

#[derive(Debug)]
pub struct CommandProtocolConfig {
    pub receiver: Receiver<InnerProtocol>,
}

#[derive(Debug)]
pub struct CommandProtocol {
    receiver: Arc<Mutex<Receiver<InnerProtocol>>>,
}

impl ProtocolHandler for CommandProtocol {
    async fn accept(&self, connection: Connection) -> std::result::Result<(), AcceptError> {
        self.handle_connection(connection)
            .await
            .map_err(|error| AcceptError::from_err(error))
    }
}

impl CommandProtocol {
    pub fn new(config: CommandProtocolConfig) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(config.receiver)),
        }
    }

    async fn handle_connection(&self, connection: Connection) -> Result<()> {
        loop {
            let input = self.receiver.lock().await.recv().await;
            match input {
                Some(InnerProtocol::NewPort { port }) => {
                    let mut send = match connection.open_uni().await {
                        Ok(streams) => streams,
                        Err(_) => return Ok(()),
                    };

                    let _ = Response::Port(port).write_to(&mut send).await;
                }
                None => {
                    println!("Euhm nothing here")
                }
            };
        }
    }
}
