//! Frontend-neutral client API intended for mobile bindings.
//!
//! This module deliberately does not depend on Ratatui, terminal I/O, or the
//! desktop IPC transport. An Android binding can own a [`MobileDaemon`] and
//! expose these operations to its UI and foreground service.

use std::{collections::VecDeque, sync::Arc, time::Instant};

use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    daemon::{Daemon, Role, Status},
    error::Result,
};

const LOG_LIMIT: usize = 200;

#[derive(Debug, Clone, Serialize)]
pub struct MobileStatus {
    pub running: bool,
    pub role: Role,
    pub serving: bool,
    pub connected: bool,
    pub forwarded_ports: Vec<u16>,
    pub available_ports: Vec<u16>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct MobileTraffic {
    pub sent: u64,
    pub received: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MobileSnapshot {
    pub status: MobileStatus,
    pub traffic: MobileTraffic,
    pub logs: Vec<String>,
}

/// Owns one client daemon for a mobile frontend.
///
/// The wrapper keeps the ticket out of status/log snapshots and makes the
/// lifetime explicit: call [`MobileDaemon::shutdown`] when the Android
/// service stops. Only the connecting role is exposed by the mobile API.
pub struct MobileDaemon {
    daemon: Arc<Daemon>,
    started: Instant,
    ticket: Mutex<Option<String>>,
    logs: Mutex<VecDeque<String>>,
}

impl MobileDaemon {
    pub async fn start() -> Result<Arc<Self>> {
        let daemon = Daemon::start().await?;
        let mobile = Arc::new(Self {
            daemon,
            started: Instant::now(),
            ticket: Mutex::new(None),
            logs: Mutex::new(VecDeque::new()),
        });
        mobile.log("devptp Android client ready").await;
        Ok(mobile)
    }

    /// Connect to a serving peer and remember the ticket for reconnect.
    pub async fn connect(&self, ticket: impl Into<String>) -> Result<()> {
        let ticket = ticket.into();
        if ticket.trim().is_empty() {
            let error = crate::error::Error::Custom("Ticket is empty".into());
            self.log("connection failed · ticket is empty").await;
            return Err(error);
        }

        self.log("connecting…").await;
        match self.daemon.start_peer(ticket.clone()).await {
            Ok(()) => {
                *self.ticket.lock().await = Some(ticket);
                self.log("connection established").await;
                Ok(())
            }
            Err(error) => {
                self.log(format!("connection failed · {error}")).await;
                Err(error)
            }
        }
    }

    /// Disconnect and connect again using the last successful ticket.
    pub async fn reconnect(&self) -> Result<()> {
        let ticket = self
            .ticket
            .lock()
            .await
            .clone()
            .ok_or_else(|| crate::error::Error::Custom("No ticket to reconnect with".into()))?;
        self.disconnect().await?;
        self.connect(ticket).await
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.daemon.disconnect().await?;
        self.log("disconnected").await;
        Ok(())
    }

    pub async fn ping(&self) -> Result<()> {
        let result = self.daemon.ping().await;
        match &result {
            Ok(()) => self.log("ping ok").await,
            Err(error) => self.log(format!("ping failed · {error}")).await,
        }
        result
    }

    pub async fn status(&self) -> MobileStatus {
        Self::status_from(self.daemon.status().await, self.started)
    }

    pub fn traffic(&self) -> MobileTraffic {
        let traffic = self.daemon.traffic();
        MobileTraffic {
            sent: traffic.sent,
            received: traffic.received,
        }
    }

    pub async fn logs(&self) -> Vec<String> {
        self.logs.lock().await.iter().cloned().collect()
    }

    pub async fn snapshot(&self) -> MobileSnapshot {
        MobileSnapshot {
            status: self.status().await,
            traffic: self.traffic(),
            logs: self.logs().await,
        }
    }

    pub async fn shutdown(&self) {
        self.daemon.shutdown().await;
        self.log("client shut down").await;
    }

    /// Access to the underlying daemon for a native binding that needs a
    /// lower-level operation. Prefer the methods on this type for new APIs.
    pub fn daemon(&self) -> &Arc<Daemon> {
        &self.daemon
    }

    async fn log(&self, message: impl Into<String>) {
        let mut logs = self.logs.lock().await;
        if logs.len() == LOG_LIMIT {
            logs.pop_front();
        }
        logs.push_back(message.into());
    }

    fn status_from(status: Status, started: Instant) -> MobileStatus {
        MobileStatus {
            running: status.running,
            role: status.role,
            serving: status.serving,
            connected: status.connected,
            forwarded_ports: status.forwarded_ports,
            available_ports: status.available_ports,
            uptime_seconds: started.elapsed().as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MobileDaemon;

    #[tokio::test]
    async fn starts_with_a_safe_snapshot() {
        let daemon = MobileDaemon::start().await.expect("mobile daemon starts");
        let snapshot = daemon.snapshot().await;

        assert!(snapshot.status.running);
        assert!(!snapshot.status.connected);
        assert!(snapshot.logs.iter().all(|line| !line.contains("ticket")));

        daemon.shutdown().await;
    }

    #[tokio::test]
    async fn empty_ticket_is_rejected() {
        let daemon = MobileDaemon::start().await.expect("mobile daemon starts");
        assert!(daemon.connect(" ").await.is_err());
        daemon.shutdown().await;
    }
}
