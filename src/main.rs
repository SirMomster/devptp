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
    },
    Status {},
    Disconnect {},
    ListForwardedPorts {},
    Shutdown {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.daemon {
        let handle = Daemon::run().await?;
        tokio::select! {
            _ = tokio::signal::ctrl_c() => handle.daemon.shutdown().await,
            _ = handle.daemon.wait_for_shutdown() => {},
        }
        return Ok(());
    }

    let client = Client::new();

    match &args.command {
        Some(Commands::Connect { ticket }) => client.send_connect(ticket.to_string()).await?,
        Some(Commands::StartServing {}) => client.send_start_serving().await?,
        Some(Commands::Ping {}) => client.send_ping().await?,
        Some(Commands::Expose { local_port }) => {
            client.send_expose(*local_port).await?;
        }
        Some(Commands::Status {}) => client.send_status().await?,
        Some(Commands::Disconnect {}) => client.send_disconnect().await?,
        Some(Commands::ListForwardedPorts {}) => client.send_list_forwarded_ports().await?,
        Some(Commands::Shutdown {}) => client.send_shutdown().await?,
        None => {
            println!("No subcomman was used");
        }
    }

    Ok(())
}
