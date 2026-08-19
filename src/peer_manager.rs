use crate::peer::Peer;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct PeerManager {
    peers: Arc<RwLock<Vec<Arc<Peer>>>>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(vec![])),
        }
    }

    pub async fn add_peer(&self, peer: Peer) {
        let arced_peer = Arc::new(peer);
        let cloned_peer = arced_peer.clone();
        {
            self.peers.write().unwrap().push(cloned_peer);
        }

        println!("Starting peer loop");
        arced_peer.run();
    }

    pub fn peers(&self) -> Vec<Arc<Peer>> {
        self.peers.read().unwrap().iter().cloned().collect()
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}
