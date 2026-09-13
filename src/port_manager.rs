use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
#[cfg(target_os = "linux")]
use std::{fs, io};

use tokio::sync::Mutex;

use crate::{config::Config, peer_manager::PeerManager, port_changes::PortChanges};

#[derive(Debug)]
pub struct PortManager {
    known_ports: Mutex<BTreeSet<u16>>,
    peer_manager: Arc<PeerManager>,
    config: Arc<Config>,
    serving: AtomicBool,
}

impl PortManager {
    pub fn new(peer_manager: Arc<PeerManager>, config: Arc<Config>) -> Self {
        Self {
            known_ports: Mutex::new(BTreeSet::new()),
            peer_manager,
            config,
            serving: AtomicBool::new(false),
        }
    }

    // handle loop to run which checkes for ports on the host system (and see which have been
    // registered)
    pub fn enable(&self) {
        self.serving.store(true, Ordering::Release);
    }

    pub fn disable(&self) {
        self.serving.store(false, Ordering::Release);
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            tokio::time::sleep(Duration::from_secs(15)).await;
            if !self.serving.load(Ordering::Acquire) {
                continue;
            }

            let port_changes = self.collect_ports().await;
            println!("Port changes: {:?}", port_changes);

            {
                *self.known_ports.lock().await = port_changes.ports_known;
            }

            let peer_manager = self.peer_manager.clone();
            if !port_changes.added.is_empty() {
                let added_allowed_ports =
                    self.config.shared_ports.intersection(&port_changes.added);
                let ports: Vec<u16> = added_allowed_ports.into_iter().copied().collect();
                if ports.is_empty() {
                    continue;
                }
                peer_manager
                    .broadcast(&crate::protocol::Message::NewKnownPorts {
                        value: ports.clone(),
                    })
                    .await;
            }

            if !port_changes.removed.is_empty() {
                let removed_allowed_ports =
                    self.config.shared_ports.intersection(&port_changes.removed);
                let ports: Vec<u16> = removed_allowed_ports.into_iter().copied().collect();
                if ports.is_empty() {
                    continue;
                }
                peer_manager
                    .broadcast(&crate::protocol::Message::NewRemovedPorts {
                        value: ports.clone(),
                    })
                    .await;
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn fetch_ports_from_file(path: &str, ports: &mut BTreeSet<u16>) -> io::Result<()> {
        let data = fs::read_to_string(path)?;

        for line in data.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 4 {
                continue;
            }

            let state = fields[3];

            if state != "0A" && state != "07" {
                continue;
            }

            if let Some(hex_port) = fields[1].split(':').nth(1)
                && let Ok(port) = u16::from_str_radix(hex_port, 16)
            {
                ports.insert(port);
            }
        }

        Ok(())
    }

    pub async fn get_allowed_known_ports(&self) -> BTreeSet<u16> {
        println!("Waiting for known_ports");
        let guard = self.known_ports.lock().await;
        println!("Acquired known_ports");

        let ports = guard.clone();
        drop(guard);

        self.config
            .shared_ports
            .intersection(&ports)
            .copied()
            .collect()
    }

    pub async fn allowed_known_ports(&self) -> Vec<u16> {
        self.get_allowed_known_ports().await.into_iter().collect()
    }

    #[cfg(target_os = "linux")]
    async fn collect_ports(&self) -> PortChanges {
        let mut ports = BTreeSet::new();

        for path in [
            "/proc/net/tcp",
            "/proc/net/tcp6",
            "/proc/net/udp",
            "/proc/net/udp6",
        ] {
            if let Err(error) = Self::fetch_ports_from_file(path, &mut ports) {
                eprintln!("Could not read {path}: {error}");
            }
        }

        let known_ports = self.known_ports.lock().await;
        PortChanges::compare_ports(&known_ports, &ports)
    }

    #[cfg(not(target_os = "linux"))]
    async fn collect_ports(&self) -> PortChanges {
        PortChanges {
            added: BTreeSet::new(),
            removed: BTreeSet::new(),
            ports_known: BTreeSet::new(),
        }
    }
}
