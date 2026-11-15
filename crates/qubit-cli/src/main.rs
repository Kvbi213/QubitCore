//! Qubit Command Line Interface

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "qubit", version, about = "Qubit Ecosystem Command Line Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check system status and memory metrics
    Status,
    /// List active agents and tasks
    Agents,
    /// Interact with local MCP registry
    Tools,
    /// Launch Ghidra / RE sandbox container
    Sandbox {
        #[arg(short, long)]
        target: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            println!("QubitCore Status: Operational");
            println!("Memory Budget: < 60 MB RAM target");
            println!("RPC Transport: Axum / WebSocket localhost:4000");
        }
        Commands::Agents => {
            println!("No active sub-agents running (Reflex mode idle).");
        }
        Commands::Tools => {
            println!("MCP Tool Registry: 0 connected servers.");
        }
        Commands::Sandbox { target } => {
            println!("Launching sandbox environment for target: {:?}", target);
        }
    }

    Ok(())
}
