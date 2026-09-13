use serde::Serialize;
use std::{sync::Arc, time::Duration};
use tokio::sync::{Mutex, Notify};

use crate::{
    dev_ptp_protocol::DevPTP,
    error::{Error, Result},
    ipc_manager::{self, IPCManager},
    peer::Peer,
    peer_manager::PeerManager,
    port_manager::PortManager,
    protocol::{ALPN, Message},
};
use iroh::{Endpoint, endpoint::presets, protocol::Router};
use iroh_tickets::endpoint::EndpointTicket;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Idle,
    Serving,
    Connecting,
}

pub struct Daemon {
    endpoint: Endpoint,
    router: Mutex<Option<Router>>,
    pub client_peer: Arc<Mutex<Option<Arc<Peer>>>>,
    peer_manager: Arc<PeerManager>,
    port_manager: Arc<PortManager>,
    pub(crate) shutdown: Arc<Notify>,
    role: Mutex<Role>,
}

pub struct DaemonHandle {
    pub daemon: Arc<Daemon>,
    pub ipc_manager: Arc<IPCManager>,
}

#[derive(Serialize)]
pub struct Status {
    pub running: bool,
    pub role: Role,
    pub serving: bool,
    pub connected: bool,
    pub ticket: Option<String>,
    pub forwarded_ports: Vec<u16>,
    pub available_ports: Vec<u16>,
}

impl Daemon {
    pub async fn run() -> Result<DaemonHandle> {
        let endpoint = Endpoint::bind(presets::N0).await?;
        endpoint.online().await;

        let peer_manager = Arc::new(PeerManager::new());
        let port_manager = Arc::new(PortManager::new(peer_manager.clone()));

        let daemon = Arc::new(Self {
            endpoint,
            router: Mutex::new(None),
            client_peer: Arc::new(Mutex::new(None)),
            peer_manager: peer_manager.clone(),
            port_manager: port_manager.clone(),
            shutdown: Arc::new(Notify::new()),
            role: Mutex::new(Role::Idle),
        });

        let ipc_manager = Arc::new(ipc_manager::IPCManager::new(daemon.clone()));

        let port_manager_clone = port_manager.clone();
        let shutdown = daemon.shutdown.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = port_manager_clone.run() => {},
                _ = shutdown.notified() => {},
            }
        });
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let ipc_manager_clone = ipc_manager.clone();
        tokio::spawn(async move {
            if let Err(error) = ipc_manager_clone.start_ipc_receiver(ready_tx).await {
                eprintln!("IPC server stopped: {error}");
            }
        });
        ready_rx
            .await
            .map_err(|_| Error::Custom("IPC server failed to start".into()))??;
        println!("Daemon started, processes running");

        let handle = DaemonHandle {
            daemon: daemon.clone(),
            ipc_manager: ipc_manager.clone(),
        };

        Ok(handle)
    }

    pub async fn start_peer(&self, ticket: String) -> Result<()> {
        if !matches!(*self.role.lock().await, Role::Idle) {
            return Err(Error::Custom(
                "connect is only available on an idle daemon".into(),
            ));
        }
        let ticket: EndpointTicket = ticket
            .parse()
            .map_err(|_| Error::Custom("Bad ticket".into()))?;

        let conn = self.endpoint.connect(ticket, ALPN).await?;

        let peer = Arc::new(Peer::new(conn, None));

        peer.wait_for(&Message::Ping).await?;
        peer.send(&Message::Pong).await?;

        println!("Peer handshake complete");

        {
            let mut client_peer = self.client_peer.lock().await;
            *client_peer = Some(peer.clone());
            *self.role.lock().await = Role::Connecting;
        }

        peer.clone().run().await;

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            println!("Requesting initial list of peers");

            if let Err(e) = peer.send(&Message::GetPorts).await {
                println!("Got an error {:?}", e);
            }
            println!("Send request");
        });

        Ok(())
    }

    pub async fn forward_local_service(&self, local_port: u16) -> Result<()> {
        if !matches!(*self.role.lock().await, Role::Serving) {
            return Err(Error::Custom(
                "expose_port is only available on a serving daemon".into(),
            ));
        }
        let peers = self.peer_manager.peers();

        for peer in peers {
            if let Err(err) = peer.send(&Message::ExposeTcp { local_port }).await {
                eprintln!("Failed to expose port on peer {}: {err}", peer.id());
            }
        }

        Ok(())
    }

    pub async fn start_receiver(&self) -> Result<()> {
        if !matches!(*self.role.lock().await, Role::Idle | Role::Serving) {
            return Err(Error::Custom(
                "A connecting daemon cannot start serving".into(),
            ));
        }
        #[cfg(not(target_os = "linux"))]
        {
            return Err(Error::Custom(
                "start_serving is only supported on Linux".to_string(),
            ));
        }

        #[cfg(target_os = "linux")]
        {
            let config = Arc::new(crate::config::get_config()?);
            self.port_manager.set_config(config);
            let mut router_ref = self.router.lock().await;
            if router_ref.is_some() {
                return Err(Error::Custom("Router already exists".to_string()));
            }

            let router = iroh::protocol::Router::builder(self.endpoint.clone())
                .accept(
                    ALPN,
                    DevPTP::new(self.peer_manager.clone(), self.port_manager.clone()),
                )
                .spawn();

            *router_ref = Some(router);
            self.port_manager.enable();
            *self.role.lock().await = Role::Serving;
        }

        Ok(())
    }

    pub async fn disconnect_port(&self, port: u16) -> Result<()> {
        if !matches!(*self.role.lock().await, Role::Connecting) {
            return Err(Error::Custom(
                "Only a connecting daemon can disconnect a forwarded port".into(),
            ));
        }
        let peer = self
            .client_peer
            .lock()
            .await
            .clone()
            .ok_or_else(|| Error::Custom("No peer is connected".into()))?;
        peer.clone().unexpose_port(port).await
    }

    pub async fn forwarded_ports(&self) -> Vec<u16> {
        match self.client_peer.lock().await.clone() {
            Some(peer) => peer.exposed_ports().await,
            None => Vec::new(),
        }
    }

    pub async fn disconnect(&self) -> Result<()> {
        if !matches!(*self.role.lock().await, Role::Connecting) {
            return Err(Error::Custom(
                "disconnect is only available on a connecting daemon".into(),
            ));
        }
        if let Some(peer) = self.client_peer.lock().await.take() {
            peer.shutdown().await;
        }
        *self.role.lock().await = Role::Idle;
        Ok(())
    }

    pub async fn status(&self) -> Status {
        let role = *self.role.lock().await;
        let serving = matches!(role, Role::Serving);
        let connected = if serving {
            self.peer_manager.len() > 0
        } else {
            self.client_peer.lock().await.is_some()
        };
        let ticket = if serving {
            self.get_ticket().await.ok()
        } else {
            None
        };
        Status {
            running: true,
            serving,
            connected,
            ticket,
            role,
            forwarded_ports: if matches!(role, Role::Connecting) {
                self.forwarded_ports().await
            } else {
                Vec::new()
            },
            available_ports: if serving {
                self.port_manager.allowed_known_ports().await
            } else {
                Vec::new()
            },
        }
    }

    pub async fn wait_for_shutdown(&self) {
        self.shutdown.notified().await;
    }

    pub async fn shutdown(&self) {
        self.port_manager.disable();
        let _ = self.disconnect().await;
        if let Some(router) = self.router.lock().await.take() {
            let _ = router.shutdown().await;
        }
        self.shutdown.notify_waiters();
        self.endpoint.close().await;
    }

    pub async fn get_ticket(&self) -> Result<String> {
        if let Some(router) = self.router.lock().await.as_ref() {
            return Ok(EndpointTicket::new(router.endpoint().addr()).to_string());
        }
        Err(Error::Custom("Router not set".to_string()))
    }
}
