use clap::{Parser, Subcommand};
use devptp::{client::Client, daemon::Daemon};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long)]
    daemon: bool,
}

#[derive(Subcommand)]
enum Commands {
    Connect {
        #[arg(long, short)]
        ticket: String,
    },
    StartServing {},
    Ping {},
    Expose {
        local_port: u16,
        remote_port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.daemon {
        let _ = Daemon::run().await;

        let _ = tokio::signal::ctrl_c().await;
        return Ok(());
    }

    let client = Client::new();

    match &args.command {
        Some(Commands::Connect { ticket }) => {
            let _ = client.send_connect(ticket.to_string()).await;
        }
        Some(Commands::StartServing {}) => {
            let _ = client.send_start_serving().await;
        }
        Some(Commands::Ping {}) => {
            let _ = client.send_ping().await;
        }
        Some(Commands::Expose {
            local_port,
            remote_port,
        }) => {
            let _ = client.send_expose(*local_port, *remote_port).await;
        }
        None => {
            println!("No subcomman was used");
        }
    }

    Ok(())
}
