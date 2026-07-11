mod error;

pub use self::error::{Error, Result};

use iroh::endpoint::{presets, Endpoint};
use iroh_tickets::endpoint::EndpointTicket;

use iroh_ping::Ping;

pub async fn run_client(ticket: EndpointTicket) -> Result<()> {
    let send_ep = Endpoint::bind(presets::N0).await?;
    let send_pinger = Ping::new();

    let rtt = send_pinger
        .ping(&send_ep, ticket.endpoint_addr().clone())
        .await
        .map_err(|e| Error::Custom(format!("Failed to send ping {:?}", e)));

    println!("Ping took: {:?} to complete", rtt);
    send_ep.close().await;
    Ok(())
}
