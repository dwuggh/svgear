use anyhow::Result;
use clap::{Parser, Subcommand};
use svgear::painter::{MathjaxServer, Mermaid};
use svgear::Painter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate SVG from MathJax
    Math {
        /// Input file path
        #[arg(short, long)]
        input: String,
    },
    /// Generate SVG from Mermaid
    Mermaid {
        /// Input
        #[arg(short, long)]
        input: String,
    },
    /// Run in server mode
    Server,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Math { input } => {
            todo!()
        }
        Commands::Mermaid { input } => {
            todo!()
        }
        Commands::Server => {
            svgear::run_server(3000).await?;
        }
    }

    Ok(())
}
