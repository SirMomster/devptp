use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::{
    error::{Error, Result},
    helpers::{receive_json, send_json},
    peer::Peer,
    peer_manager::PeerManager,
    protocol::{Message, Request, Response, ALPN, STREAM_NAME},
};
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericNamespaced, ListenerOptions,
};
use iroh::{endpoint::presets, protocol::Router, Endpoint};
use iroh_tickets::endpoint::EndpointTicket;
use std::io;
use tokio::io::BufReader;

#[derive(Debug)]
pub struct DevPTP {
    peer_manager: Arc<PeerManager>,
}

pub struct Daemon {
    endpoint: Endpoint,
    router: Mutex<Option<Router>>,
    client_peer: Arc<Mutex<Option<Arc<Peer>>>>,
    peer_manager: Arc<PeerManager>,
}

impl Daemon {
    pub async fn run() -> Result<Arc<Self>> {
        let endpoint = Endpoint::bind(presets::N0).await?;
        endpoint.online().await;

        let daemon = Arc::new(Self {
            endpoint,
            router: Mutex::new(None),
            client_peer: Arc::new(Mutex::new(None)),
            peer_manager: Arc::new(PeerManager::new()),
        });

        let arc_clone = daemon.clone();
        tokio::spawn(async move {
            let _ = arc_clone.start_ipc_receiver().await;
        });
        Ok(daemon)
    }

    async fn start_ipc_receiver(self: Arc<Self>) -> Result<()> {
        let name = STREAM_NAME.to_ns_name::<GenericNamespaced>()?;

        let listener = ListenerOptions::new().name(name).create_tokio()?;

        println!("IPC server listening");
        loop {
            let stream = listener.accept().await?;
            let value = self.clone();
            tokio::spawn(async move {
                if let Err(error) = value.handle_ipc_client(stream).await {
                    eprintln!("Client error: {error}");
                }
            });
        }
    }

    async fn handle_ipc_client(self: Arc<Self>, stream: Stream) -> io::Result<()> {
        let (read_half, mut write_half) = tokio::io::split(stream);
        let mut reader = BufReader::new(read_half);

        while let Some(request) = receive_json::<_, Request>(&mut reader).await? {
            let response = match request {
                Request::Ping => {
                    let client_peer = self.client_peer.clone();
                    let guard = client_peer.lock().await;

                    if let Some(peer) = guard.as_ref() {
                        match peer.send(&crate::protocol::Message::Ping).await {
                            Ok(_) => Response::Pong,
                            Err(e) => Response::Error {
                                message: e.to_string(),
                            },
                        }
                    } else {
                        Response::Error {
                            message: "Bad bad output".to_string(),
                        }
                    }
                }
                Request::StartServing => match self.start_receiver().await {
                    Ok(_) => Response::ServingStarted {
                        ticket: self.get_ticket().await.expect("I expect a ticket"),
                    },
                    Err(e) => Response::Error {
                        message: e.to_string(),
                    },
                },
                Request::ExposePort { local_port } => {
                    let daemon = self.clone();

                    tokio::spawn(async move {
                        if let Err(err) = daemon.forward_local_service(local_port).await {
                            eprintln!("Port exposure failed: {err}");
                        }
                    });
                    Response::Ok
                }
                Request::Connect { ticket } => match self.start_peer(ticket).await {
                    Ok(()) => Response::Ok,
                    Err(e) => Response::Error {
                        message: e.to_string(),
                    },
                },
            };

            send_json(&mut write_half, &response).await?;
        }

        Ok(())
    }

    async fn start_peer(&self, ticket: String) -> Result<()> {
        let ticket: EndpointTicket = ticket
            .parse()
            .map_err(|_| Error::Custom("Bad ticket".into()))?;

        let conn = self.endpoint.connect(ticket, ALPN).await?;

        let peer = Arc::new(Peer::new(conn));

        peer.wait_for(&Message::Ping).await?;
        peer.send(&Message::Pong).await?;

        println!("Peer handshake complete");

        {
            let mut client_peer = self.client_peer.lock().await;
            *client_peer = Some(peer.clone());
        }

        peer.run();

        Ok(())
    }

    pub async fn forward_local_service(&self, local_port: u16) -> Result<()> {
        let peers = self.peer_manager.peers();

        for peer in peers {
            if let Err(err) = peer.send(&Message::ExposeTcp { local_port }).await {
                eprintln!("Failed to expose port on peer {}: {err}", peer.id());
            }
        }

        Ok(())
    }

    async fn start_receiver(&self) -> Result<()> {
        let mut router_ref = self.router.lock().await;

        if router_ref.is_some() {
            return Err(Error::Custom("Router already exists".to_string()));
        }

        let router = iroh::protocol::Router::builder(self.endpoint.clone())
            .accept(ALPN, DevPTP::new(self.peer_manager.clone()))
            .spawn();

        *router_ref = Some(router);

        Ok(())
    }

    pub async fn get_ticket(&self) -> Result<String> {
        if let Some(router) = self.router.lock().await.as_ref() {
            return Ok(EndpointTicket::new(router.endpoint().addr()).to_string());
        }
        Err(Error::Custom("Router not set".to_string()))
    }
}

impl DevPTP {
    pub fn new(peer_manager: Arc<PeerManager>) -> Self {
        Self { peer_manager }
    }

    async fn handshake(&self, peer: &Peer) -> crate::error::Result<()> {
        peer.send(&crate::protocol::Message::Ping).await?;
        peer.wait_for(&crate::protocol::Message::Pong).await?;
        Ok(())
    }
}

impl iroh::protocol::ProtocolHandler for DevPTP {
    async fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> std::result::Result<(), iroh::protocol::AcceptError> {
        let peer = Peer::new(connection);

        self.handshake(&peer)
            .await
            .map_err(iroh::protocol::AcceptError::from_err)?;

        self.peer_manager.add_peer(peer).await;

        Ok(())
    }
}
