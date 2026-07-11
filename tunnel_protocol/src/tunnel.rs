use iroh::{
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
    EndpointId,
};

/// Error type for tunnel operations.
#[derive(Debug)]
pub enum TunnelError {
    EmptyPayload,
}

impl std::fmt::Display for TunnelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPayload => write!(f, "empty payload"),
        }
    }
}

impl std::error::Error for TunnelError {}

/// The ALPN identifier for the tunnel protocol.
pub const TUNNEL_ALPN: &[u8] = b"devptp/tunnel/0";

/// Tunnel protocol handler with echo support.
///
/// When `echo` is true, incoming data is echoed back to the sender.
/// When `echo` is false, the connection is accepted but no data is processed
/// (placeholder for future port tunneling logic).
pub struct Tunnel {
    echo: bool,
}

impl Tunnel {
    /// Create a new Tunnel handler.
    pub fn new(echo: bool) -> Self {
        Self { echo }
    }
}

impl ProtocolHandler for Tunnel {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let node_id = connection.remote_id();
        println!("tunnel: accepted connection from {node_id}");

        // The client opens a bidirectional stream.
        let (mut send, mut recv) = connection.accept_bi().await?;

        if self.echo {
            // Echo mode: read all data from the client and send it back.
            let buf = recv
                .read_to_end(usize::MAX)
                .await
                .map_err(AcceptError::from_err)?;

            if buf.is_empty() {
                return Err(AcceptError::from_err(TunnelError::EmptyPayload));
            }

            // Send back the same data as a response.
            send.write_all(&buf).await.map_err(AcceptError::from_err)?;

            // Signal we're done sending.
            send.finish()?;

            // Wait for the remote to close the connection.
            connection.closed().await;
        } else {
            // Placeholder: no-op, ready for future port tunneling logic.
            // Close the connection immediately.
            connection.close(0u32.into(), b"");
        }

        Ok(())
    }
}

impl std::fmt::Debug for Tunnel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tunnel").field("echo", &self.echo).finish()
    }
}

/// Result of a tunnel operation.
#[derive(Debug)]
pub struct TunnelResult {
    pub node_id: EndpointId,
    pub payload: Vec<u8>,
}
