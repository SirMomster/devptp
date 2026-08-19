use tokio::sync::broadcast::Receiver;

use crate::incoming_commands::IncomingCommand;
use crate::port_forwarder::PortForwarder;
use crate::error::Result;

pub struct CommandHandler {
    connection: iroh::endpoint::Connection,
    rx: Receiver<IncomingCommand>,
}

impl CommandHandler {
    pub fn new(connection: iroh::endpoint::Connection, rx: Receiver<IncomingCommand>) -> Self {
        Self { connection, rx }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            let cmd = match self.rx.recv().await {
                Ok(cmd) => cmd,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    println!("Dropped {} incoming commands", n);
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    println!("Broadcast channel closed");
                    break;
                }
            };

            match cmd {
                IncomingCommand::ProposedPort { port } => {
                    println!("Received proposed port: {}", port);

                    // Pick a local port dynamically by binding to 0
                    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
                        Ok(l) => l,
                        Err(e) => {
                            eprintln!("Failed to bind local port: {:?}", e);
                            continue;
                        }
                    };
                    let local_port = listener.local_addr().unwrap().port();
                    println!(
                        "Opened port forward 127.0.0.1:{} → receiver port {}",
                        local_port, port
                    );

                    let conn = self.connection.clone();
                    tokio::spawn(async move {
                        if let Err(e) = PortForwarder::new(conn, local_port, port).run().await {
                            eprintln!("Port forward error: {:?}", e);
                        }
                    });
                }
                IncomingCommand::Ok => {
                    println!("Got ACK!");
                }
            }
        }

        Ok(())
    }
}
