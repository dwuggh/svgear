use anyhow::Result;
use clap::{Parser, Subcommand};
use svgear::Painter;
use svgear::painter::{Mermaid, MathjaxServer};
use std::fs;
use std::path::PathBuf;

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
            let content = fs::read_to_string(input)?;
            let painter = MathjaxServer;
            let svg = painter.paint(&content)?;
            fs::write(output, svg)?;
        }
        Commands::Mermaid { input } => {
            let content = fs::read_to_string(input)?;
            let painter = Mermaid;
            let svg = painter.paint(&content)?;
            fs::write(output, svg)?;
        }
        Commands::Server => {
            svgear::run_server().await?;
        }
    }

    Ok(())
}
