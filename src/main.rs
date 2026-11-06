use clap::{Parser, Subcommand};
use devptp::{client::Client, daemon::Daemon};

#[derive(Parser)]
#[command(
    version,
    about = concat!("Peer-to-peer development networking utility v", env!("CARGO_PKG_VERSION")),
    after_long_help = "Configure the relay with `relay_url = \"https://relay.example.com\"` in your TOML config. The active relay URL is shown by `devptp status`."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long)]
    daemon: bool,

    /// Use this configuration file instead of normal config-path resolution.
    #[arg(long, value_name = "PATH")]
    config_file: Option<std::path::PathBuf>,
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
    #[command(about = "Show daemon status, including the active relay URL")]
    Status {},
    Disconnect {
        #[arg(long)]
        port: Option<u16>,
    },
    ListForwardedPorts {},
    Shutdown {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.daemon {
        let handle = Daemon::run_with_config(args.config_file.as_deref()).await?;
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
        Some(Commands::Disconnect { port }) => client.send_disconnect(*port).await?,
        Some(Commands::ListForwardedPorts {}) => client.send_list_forwarded_ports().await?,
        Some(Commands::Shutdown {}) => client.send_shutdown().await?,
        None => {
            println!("No subcomman was used");
        }
    }

    Ok(())
}
