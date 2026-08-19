use std::{collections::HashMap, sync::Arc};

use crate::{
    error::{Error, Result},
    helpers::{receive_json, send_json},
    protocol::Message,
};

use iroh::{
    endpoint::{Connection, RecvStream, SendStream},
    EndpointId,
};

use tokio::{
    io::{AsyncRead, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::JoinHandle,
};

#[derive(Debug)]
pub struct Peer {
    id: EndpointId,
    connection: Connection,
    exposed_ports: Mutex<HashMap<u16, JoinHandle<()>>>,
}

impl Peer {
    pub fn new(connection: Connection) -> Self {
        Self {
            id: connection.remote_id(),
            connection,
            exposed_ports: Mutex::new(HashMap::new()),
        }
    }

    pub fn id(&self) -> EndpointId {
        self.id
    }

    /*
     * Uni streams
     */

    pub async fn send(&self, message: &Message) -> Result<()> {
        let bytes = serde_json::to_vec(message).map_err(|e| Error::Custom(e.to_string()))?;

        let mut stream = self.connection.open_uni().await?;

        stream.write_all(&bytes).await?;
        stream.finish()?;

        Ok(())
    }

    pub async fn receive(&self) -> Result<Message> {
        let mut stream = self.connection.accept_uni().await?;

        let bytes = stream.read_to_end(1024 * 1024).await?;

        serde_json::from_slice(&bytes).map_err(|e| Error::Custom(format!("Parse error: {e}")))
    }

    pub async fn wait_for(&self, expected: &Message) -> Result<Message> {
        let message = self.receive().await?;

        if message != *expected {
            return Err(Error::Custom(format!(
                "Expected {expected:?}, got {message:?}"
            )));
        }

        Ok(message)
    }

    /*
     * Peer loops
     */

    pub fn run(self: Arc<Self>) {
        self.clone().run_uni_loop();
        self.run_bi_loop();
    }

    fn run_uni_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                let message = match self.receive().await {
                    Ok(message) => message,

                    Err(err) => {
                        eprintln!("Peer {} uni loop stopped: {err}", self.id());
                        break;
                    }
                };

                match message {
                    Message::Ping => {
                        println!("Got ping from {}", self.id());
                    }

                    Message::Pong => {
                        println!("Got pong from {}", self.id());
                    }

                    Message::Text { value } => {
                        println!("Got text from {}: {value}", self.id());
                    }

                    Message::ExposeTcp { local_port } => {
                        let inner = self.clone();
                        if let Err(e) = inner.expose_port(local_port).await {
                            println!("Failed to expose port {}", e);
                        };
                    }
                    Message::UnexposeTcp { local_port } => {
                        let inner = self.clone();
                        if let Err(e) = inner.unexpose_port(local_port).await {
                            println!("Failed to unexpose port {}", e);
                        }
                    }
                    Message::ForwardTcp { .. } => {
                        eprintln!("Received ForwardTcp on uni stream; ignoring");
                    }
                }
            }
        });
    }

    pub async fn expose_port(self: Arc<Self>, local_port: u16) -> Result<()> {
        {
            let ports = self.exposed_ports.lock().await;

            if ports.contains_key(&local_port) {
                return Err(Error::Custom(format!(
                    "Port {local_port} is already exposed"
                )));
            }
        }

        let peer = self.clone();

        let handle = tokio::spawn(async move {
            if let Err(err) = peer.serve_forwarded_port(local_port).await {
                eprintln!("Forwarded port {local_port} stopped: {err}");
            }
        });

        self.exposed_ports.lock().await.insert(local_port, handle);

        Ok(())
    }

    pub async fn unexpose_port(self: Arc<Self>, remote_port: u16) -> Result<()> {
        let handle = self.exposed_ports.lock().await.remove(&remote_port);

        match handle {
            Some(handle) => {
                handle.abort();
                Ok(())
            }

            None => Err(Error::Custom(format!("Port {remote_port} is not exposed"))),
        }
    }

    fn run_bi_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                let stream = match self.connection.accept_bi().await {
                    Ok(stream) => stream,

                    Err(err) => {
                        eprintln!("Peer {} bi loop stopped: {err}", self.id());
                        break;
                    }
                };

                let peer = self.clone();

                tokio::spawn(async move {
                    if let Err(err) = peer.handle_bi_stream(stream).await {
                        eprintln!("Peer {} bi stream failed: {err}", peer.id());
                    }
                });
            }
        });
    }

    /*
     * Client-side exposed listener
     */

    async fn serve_forwarded_port(self: Arc<Self>, local_port: u16) -> Result<()> {
        let remote_port = 8080; // TODO: either use local or a available port
        let listener = TcpListener::bind(("127.0.0.1", remote_port)).await?;

        println!("Forwarding 127.0.0.1:{remote_port} -> peer -> 127.0.0.1:{local_port}");

        loop {
            let (tcp, addr) = listener.accept().await?;

            println!("Accepted connection on port {remote_port} from {addr}");

            let peer = self.clone();

            tokio::spawn(async move {
                if let Err(err) = peer.open_forward_stream(tcp, local_port).await {
                    eprintln!("TCP tunnel failed: {err}");
                }
            });
        }
    }

    /*
     * Open one tunnel
     */

    async fn open_forward_stream(&self, tcp: TcpStream, local_port: u16) -> Result<()> {
        let (mut send, recv) = self.connection.open_bi().await?;

        // This header identifies what this new stream is for.
        send_json(&mut send, &Message::ForwardTcp { port: local_port }).await?;

        Self::bridge_tcp(send, recv, tcp).await
    }

    /*
     * Handle one incoming tunnel
     */

    async fn handle_bi_stream(&self, stream: (SendStream, RecvStream)) -> Result<()> {
        let (send, recv) = stream;

        // Keep this BufReader alive after reading the header so we
        // don't lose any already-buffered TCP payload.
        let mut recv = BufReader::new(recv);

        let message = receive_json::<_, Message>(&mut recv)
            .await?
            .ok_or_else(|| Error::Custom("Bi stream closed before header".into()))?;

        match message {
            Message::ForwardTcp { port } => {
                println!("Opening daemon-side TCP connection to 127.0.0.1:{port}");

                let tcp = TcpStream::connect(("127.0.0.1", port)).await?;

                Self::bridge_tcp(send, recv, tcp).await
            }

            other => Err(Error::Custom(format!(
                "Unexpected bi-stream message: {other:?}"
            ))),
        }
    }

    /*
     * Actual byte bridge
     */

    async fn bridge_tcp<R>(mut send: SendStream, mut recv: R, tcp: TcpStream) -> Result<()>
    where
        R: AsyncRead + Unpin,
    {
        let (mut tcp_read, mut tcp_write) = tcp.into_split();

        let iroh_to_tcp = async {
            tokio::io::copy(&mut recv, &mut tcp_write).await?;

            tcp_write.shutdown().await?;

            Ok::<(), Error>(())
        };

        let tcp_to_iroh = async {
            tokio::io::copy(&mut tcp_read, &mut send).await?;

            send.finish()?;

            Ok::<(), Error>(())
        };

        tokio::try_join!(iroh_to_tcp, tcp_to_iroh)?;

        Ok(())
    }
}
