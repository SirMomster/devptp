use super::super::error::{Error, Result};

use iroh::{
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};

use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::tunnel_protocol::tunnel::request::Request;
use crate::tunnel_protocol::tunnel::response::Response;

pub const TUNNEL_ALPN: &[u8] = b"devptp/tcp-tunnel/0";

async fn proxy(
    mut send: iroh::endpoint::SendStream,
    mut recv: BufReader<iroh::endpoint::RecvStream>,
    tcp: TcpStream,
) -> Result<()> {
    let (mut tcp_read, mut tcp_write) = tcp.into_split();

    let client_to_tcp = async {
        let copied = tokio::io::copy(&mut recv, &mut tcp_write).await?;
        tcp_write.shutdown().await?;

        Ok::<u64, Error>(copied)
    };

    let tcp_to_client = async {
        let copied = tokio::io::copy(&mut tcp_read, &mut send).await?;
        send.finish()?;

        Ok::<u64, Error>(copied)
    };

    let (client_bytes, server_bytes) = tokio::try_join!(client_to_tcp, tcp_to_client)?;

    println!("Proxy closed: client -> TCP: {client_bytes}, TCP -> client: {server_bytes}");

    Ok(())
}

#[derive(Debug, Clone)]
pub struct TunnelConfig {
    pub allowed_ports: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct TunnelProtocol {
    allowed_ports: Vec<u16>,
}

impl TunnelProtocol {
    pub fn new(config: &TunnelConfig) -> Self {
        Self {
            allowed_ports: config.allowed_ports.clone(),
        }
    }

    fn port_allowed(&self, port: u16) -> bool {
        self.allowed_ports.contains(&port)
    }
}

impl ProtocolHandler for TunnelProtocol {
    async fn accept(&self, connection: Connection) -> std::result::Result<(), AcceptError> {
        self.handle_connection(connection)
            .await
            .map_err(|error| AcceptError::from_err(error))
    }
}

impl TunnelProtocol {
    async fn handle_connection(&self, connection: Connection) -> Result<()> {
        loop {
            let (send, recv) = match connection.accept_bi().await {
                Ok(streams) => streams,
                Err(_) => return Ok(()),
            };

            let tunnel = self.clone();

            tokio::spawn(async move {
                if let Err(_error) = tunnel.handle_stream(send, recv).await {
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
