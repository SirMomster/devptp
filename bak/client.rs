use iroh::endpoint::{presets, Endpoint};
use iroh_tickets::endpoint::EndpointTicket;
use iroh_ping::Ping;
use crate::tunnel_protocol::{command, tunnel};
use super::error::{Error, Result};
use crate::command_handler::CommandHandler;
use crate::incoming_commands::{IncomingCommand, IncomingCommands};

pub async fn run_client(ticket: EndpointTicket) -> Result<()> {
    let endpoint = Endpoint::bind(presets::N0).await?;
    let send_pinger = Ping::new();

    let rtt = send_pinger
        .ping(&endpoint, ticket.endpoint_addr().clone())
        .await
        .map_err(|e| Error::Custom(format!("Failed to send ping {:?}", e)));

    println!("Ping took: {:?} to complete", rtt);
    println!("Starting tunnel interface");
    let (tx, rx) = tokio::sync::broadcast::channel::<IncomingCommand>(1);
    let connection = endpoint
        .connect(ticket.endpoint_addr().clone(), tunnel::TUNNEL_ALPN)
        .await
        .map_err(|e| Error::ConnectError(e))?;

    let command_connection = endpoint
        .connect(ticket.endpoint_addr().clone(), command::COMMAND_ALPN)
        .await
        .map_err(|e| Error::ConnectError(e))?;

    let command_thread = tokio::spawn(async move {
        let _ = IncomingCommands::new(command_connection, tx).run().await;
    });

    let handler_thread = tokio::spawn(async move {
        if let Err(e) = CommandHandler::new(connection, rx).run().await {
            eprintln!("Command handler error: {:?}", e);
        };
    });

    tokio::join!(command_thread, handler_thread);

    endpoint.close().await;
    Ok(())
}
