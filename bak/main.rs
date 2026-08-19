use anyhow::Result;
use clap::{Parser, Subcommand};
use iroh_tickets::endpoint::EndpointTicket;
use tokio::signal;

mod receiver;
mod client;
mod error;
mod proc_net_tcp;
mod command_handler;
mod incoming_commands;
mod port_forwarder;

mod tunnel_protocol;

/// devptp — peer-to-peer networking toolkit built on iroh
#[derive(Parser)]
#[command(
    name = "devptp",
    about = "P2P networking toolkit for endpoint discovery and ping"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a receiver: bind an endpoint, print ticket, wait for connections
    Receiver,
    /// Ping a receiver using its ticket
    Sender {
        /// The ticket string from a receiver
        ticket: String,
        tui: Option<bool>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Receiver => {
            let allowed_ports: Vec<u16> = vec![8080, 9090];
            let result = receiver::run_receiver(allowed_ports).await?;
            println!("Connected: {}", result.ticket);
            signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl-c event");
            result.router.endpoint().close().await;
            Ok(())
        }
        Commands::Sender { ticket, tui: _ } => {
            let ticket: EndpointTicket = ticket
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid ticket: {}", e))?;
            client::run_client(ticket).await?;
            Ok(())
        }
    }
}
