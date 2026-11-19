//! QubitCore Daemon Entry Point

use clap::Parser;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "qubit-core", version, about = "QubitCore Distributed Agent OS Daemon")]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:4000")]
    bind: SocketAddr,

    #[arg(short, long, default_value = "qubit_data.db")]
    db: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    qubit_core::start_server(args.bind, &args.db).await
}
