use iroh::endpoint::Connection;
use tokio::io::BufReader;
use tokio::sync::broadcast::Sender;

use crate::tunnel_protocol::command::Response;

#[derive(Debug, Clone)]
pub enum IncomingCommand {
    ProposedPort { port: u16 },
    Ok,
}

pub struct IncomingCommands {
    connection: Connection,
    tx: Sender<IncomingCommand>,
}

impl IncomingCommands {
    pub fn new(connection: Connection, tx: Sender<IncomingCommand>) -> Self {
        Self { connection, tx }
    }

    pub async fn run(self) {
        loop {
            let connection = self.connection.clone();
            println!("Awaiting incoming connection!");
            let recv = match connection.accept_uni().await {
                Ok(recv) => recv,
                Err(e) => {
                    println!("Got an error on uni accept: {:?}", e);
                    continue;
                }
            };
            println!("Accepted incoming connection!");
            let mut recv = BufReader::new(recv);

            let result = Response::read_from(&mut recv).await;
            match result {
                Ok(response) => {
                    println!("Got a good message: {:?}", response);
                    // Forward the response via broadcast channel as an IncomingCommand
                    let incoming = match response {
                        Response::Ok => IncomingCommand::Ok,
                        Response::Error { message } => {
                            println!("Got an error response: {}", message);
                            continue;
                        }
                        Response::Port(port) => IncomingCommand::ProposedPort { port },
                    };
                    if let Err(e) = self.tx.send(incoming) {
                        eprintln!("Failed to send response via broadcast: {:?}", e);
                    }
                }
                Err(e) => {
                    println!("Got an error: {:?}", e);
                }
            }
        }
    }
}
