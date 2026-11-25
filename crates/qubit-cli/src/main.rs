use clap::{Parser, Subcommand};
use qubit_cli::{run_cli, CliCommand};

#[derive(Parser)]
#[command(name = "qubit", version, about = "QubitCore CLI client")]
struct Cli {
    #[arg(long, env = "QUBIT_URL", default_value = "http://127.0.0.1:4000")]
    url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check QubitCore daemon health and RSS memory
    Status,
    /// Mount current working directory as an MCP Root (`.`)
    #[command(name = ".")]
    MountDot,
    /// Send a prompt to QubitCore
    Chat {
        prompt: String,
        #[arg(long)]
        deep: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cmd = match cli.command {
        Commands::Status => CliCommand::Status,
        Commands::MountDot => CliCommand::MountCurrentDir,
        Commands::Chat { prompt, deep } => CliCommand::Chat { prompt, deep },
    };

    match run_cli(cmd, &cli.url).await {
        Ok(out) => {
            println!("{out}");
            Ok(())
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
