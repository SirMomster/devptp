use crate::{peer::Peer, protocol::Message};
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
        arced_peer.run().await;
    }

    pub async fn broadcast(&self, message: &Message) {
        let mut join_set = tokio::task::JoinSet::new();

        for peer in self.peers() {
            let message = message.clone();

            join_set.spawn(async move { peer.send(&message).await });
        }

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(_send_result) => {
                    // Handle send_result if Peer::send returns a Result.
                    println!("Unable to send message because of error on send");
                }
                Err(_join_error) => {
                    // The task panicked or was cancelled.
                    println!("Unable to send message because of join error");
                }
            }
        }
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
