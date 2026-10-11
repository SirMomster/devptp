//! Core devptp runtime and frontend-independent operations.
//!
//! [`daemon::Daemon::start`] starts the networking runtime without opening IPC.
//! Use [`daemon::Daemon::start_ipc`] when a frontend also needs to serve the
//! standard local IPC protocol.

pub use daemon::{Daemon, DaemonHandle, Role, Status};
pub use error::{Error, Result};

pub mod client;
pub mod config;
pub mod daemon;
pub mod dev_ptp_protocol;
pub mod error;
pub mod helpers;
pub mod ipc_manager;
pub mod mediator;
pub mod peer;
pub mod peer_manager;
pub mod port_changes;
pub mod port_manager;
pub mod protocol;
