use crate::Result;

use iroh::{
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};

use tokio::{io::BufReader, net::TcpStream};

use crate::request::Request;
use crate::response::Response;

pub const TUNNEL_ALPN: &[u8] = b"devptp/tcp-tunnel/0";

async fn proxy(
    mut send: iroh::endpoint::SendStream,
    mut recv: BufReader<iroh::endpoint::RecvStream>,
    tcp: TcpStream,
) -> Result<()> {
    let (mut tcp_read, mut tcp_write) = tcp.into_split();

    let client_to_tcp = tokio::io::copy(&mut recv, &mut tcp_write);
    let tcp_to_client = tokio::io::copy(&mut tcp_read, &mut send);

    tokio::try_join!(client_to_tcp, tcp_to_client)?;

    send.finish()?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Tunnel {
    allowed_ports: Vec<u16>,
}

impl Tunnel {
    pub fn new(allowed_ports: Vec<u16>) -> Self {
        Self { allowed_ports }
    }

    fn port_allowed(&self, port: u16) -> bool {
        self.allowed_ports.contains(&port)
    }
}

impl ProtocolHandler for Tunnel {
    async fn accept(&self, connection: Connection) -> std::result::Result<(), AcceptError> {
        self.handle_connection(connection)
            .await
            .map_err(|error| AcceptError::from_err(error))
    }
}

impl Tunnel {
    async fn handle_connection(&self, connection: Connection) -> Result<()> {
        loop {
            let (send, recv) = match connection.accept_bi().await {
                Ok(streams) => streams,
                Err(_) => return Ok(()),
            };

            let tunnel = self.clone();

            tokio::spawn(async move {
                if let Err(error) = tunnel.handle_stream(send, recv).await {
                    println!("Tunnel stream failed");
                }
            });
        }
    }

    async fn handle_stream(
        &self,
        mut send: iroh::endpoint::SendStream,
        recv: iroh::endpoint::RecvStream,
    ) -> Result<()> {
        let mut recv = BufReader::new(recv);

        let request = Request::read_from(&mut recv).await?;

        match request {
            Request::Open { port } => {
                if !self.port_allowed(port) {
                    Response::Error {
                        message: format!("port {port} is not allowed"),
                    }
                    .write_to(&mut send)
                    .await?;

                    send.finish()?;

                    return Ok(());
                }

                let tcp = match TcpStream::connect(("127.0.0.1", port)).await {
                    Ok(tcp) => tcp,
                    Err(error) => {
                        Response::Error {
                            message: error.to_string(),
                        }
                        .write_to(&mut send)
                        .await?;

                        send.finish()?;
                        return Ok(());
                    }
                };

                Response::Ok.write_to(&mut send).await?;

                proxy(send, recv, tcp).await?;
            }
        }

        Ok(())
    }
}
