mod error;

pub use self::error::{Error, Result};

use iroh::{endpoint::presets, protocol::Router, Endpoint};
use iroh_ping::Ping;
use iroh_tickets::endpoint::EndpointTicket;

use tunnel_protocol::tunnel;

pub struct ReceiverResult {
    pub ticket: String,
    pub router: Router,
}

pub async fn run_receiver() -> Result<ReceiverResult> {
    let endpoint = Endpoint::bind(presets::N0).await?;
    endpoint.online().await;

    let ping = Ping::new();
    let tunnel_recv = tunnel::Tunnel::new(vec![8000]);
    let ticket = EndpointTicket::new(endpoint.addr());

    // Spawn the ping router.
    let router = Router::builder(endpoint.clone())
        .accept(iroh_ping::ALPN, ping)
        .accept(tunnel::TUNNEL_ALPN, tunnel_recv)
        .spawn();

    Ok(ReceiverResult {
        ticket: ticket.to_string(),
        router,
    })
}
