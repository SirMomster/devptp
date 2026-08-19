use super::error::Result;
use iroh::{endpoint::presets, protocol::Router, Endpoint};
use iroh_ping::Ping;
use iroh_tickets::endpoint::EndpointTicket;
use tokio::sync::mpsc::channel;
use crate::tunnel_protocol::{
    command::{self, InnerProtocol},
    tunnel,
};

pub struct ReceiverResult {
    pub ticket: String,
    pub router: Router,
    pub sender: tokio::sync::mpsc::Sender<InnerProtocol>,
}

pub async fn run_receiver(allowed_ports: Vec<u16>) -> Result<ReceiverResult> {
    let endpoint = Endpoint::bind(presets::N0).await?;
    endpoint.online().await;

    let ping = Ping::new();
    let tunnel_config = tunnel::TunnelConfig {
        allowed_ports: allowed_ports.clone(),
    };

    let (sender, receiver) = channel::<InnerProtocol>(1);
    let command_config = command::CommandProtocolConfig { receiver };

    let tunnel_recv = tunnel::TunnelProtocol::new(&tunnel_config);
    let command_recv = command::CommandProtocol::new(command_config);
    let ticket = EndpointTicket::new(endpoint.addr());

    // Spawn the ping router.
    let router = Router::builder(endpoint.clone())
        .accept(iroh_ping::ALPN, ping)
        .accept(tunnel::TUNNEL_ALPN, tunnel_recv)
        .accept(command::COMMAND_ALPN, command_recv)
        .spawn();

    Ok(ReceiverResult {
        ticket: ticket.to_string(),
        router,
        sender,
    })
}
