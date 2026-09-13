use std::sync::Arc;

use crate::{peer::Peer, peer_manager::PeerManager, port_manager::PortManager};

#[derive(Debug)]
pub struct DevPTP {
    peer_manager: Arc<PeerManager>,
    port_manager: Arc<PortManager>,
}

impl DevPTP {
    pub fn new(peer_manager: Arc<PeerManager>, port_manager: Arc<PortManager>) -> Self {
        Self {
            peer_manager,
            port_manager,
        }
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
        let peer = Peer::new(connection, Some(self.port_manager.clone()));

        self.handshake(&peer)
            .await
            .map_err(iroh::protocol::AcceptError::from_err)?;

        self.peer_manager.add_peer(peer).await;

        Ok(())
    }
}
