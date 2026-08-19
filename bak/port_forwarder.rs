use iroh::endpoint::Connection;
use tokio::io::{self, BufReader};
use tokio::net::TcpListener;

use crate::error::Result;

use crate::tunnel_protocol::tunnel;

pub struct PortForwarder {
    connection: Connection,
    local_port: u16,
    remote_port: u16,
}

impl PortForwarder {
    pub fn new(connection: Connection, local_port: u16, remote_port: u16) -> Self {
        Self {
            connection,
            local_port,
            remote_port,
        }
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(("127.0.0.1", self.local_port)).await?;

        loop {
            let connection = self.connection.clone();
            let (mut local_tcp, peer_addr) = listener.accept().await?;

            println!(
                "Forwarding 127.0.0.1:{} to receiver port {}",
                self.local_port, self.remote_port
            );

            tokio::spawn(async move {
                let result: Result<()> = async {
                    let (mut send, recv) = connection.open_bi().await?;

                    tunnel::Request::Open { port: self.remote_port }
                        .write_to(&mut send)
                        .await?;

                    let mut recv = BufReader::new(recv);

                    match tunnel::Response::read_from(&mut recv).await? {
                        tunnel::Response::Ok => {}
                        response => {
                            return Err(crate::error::Error::Custom(format!(
                                "Remote rejected tunnel request: {response:?}"
                            )))
                        }
                    };

                    let mut remote_stream = io::join(recv, send);

                    let (tcp_to_remote, remote_to_tcp) =
                        io::copy_bidirectional(&mut local_tcp, &mut remote_stream).await?;
                    println!(
                        "Tunnel for {peer_addr} closed: \
                         TCP -> remote: {tcp_to_remote} bytes, \
                         remote -> TCP: {remote_to_tcp} bytes"
                    );
                    Ok(())
                }
                .await;
                if let Err(error) = result {
                    eprintln!("Tunnel error for {peer_addr}: {error:?}");
                }
            });
        }
    }
}
