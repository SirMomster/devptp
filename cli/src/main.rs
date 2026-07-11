use anyhow::anyhow;
use anyhow::Result;
use iroh_tickets::endpoint::EndpointTicket;

use std::env;
use tokio::signal;

use cdevptp::run_client;
use rdevptp::run_receiver;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut args = env::args().skip(1);

    let role = args
        .next()
        .ok_or_else(|| anyhow!("Expected 'receiver' or 'sender' as the first argument"))?;

    match role.as_str() {
        "receiver" => {
            let result = run_receiver().await?;
            println!("Connected: {}", result.ticket);
            signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl-c event");
            result.router.endpoint().close().await;
            Ok(())
        }
        "sender" => {
            let ticket_string = args
                .next()
                .ok_or_else(|| anyhow!("Expected a ticket string as the first parameter"))?;

            let ticket: EndpointTicket = ticket_string.parse().unwrap();
            run_client(ticket).await?;
            Ok(())
        }
        _ => Err(anyhow!(
            "Unknown role '{}'; use 'receiver' or 'sender'",
            role
        )),
    }
}
