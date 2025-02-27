use anyhow::Result;
use clap::{Parser, Subcommand};
use svgear::{HttpPainter, MermaidPainter, Painter};
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
        input: PathBuf,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Generate SVG from Mermaid
    Mermaid {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Run in server mode
    Server,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Math { input, output } => {
            let content = fs::read_to_string(input)?;
            let painter = HttpPainter;
            let svg = painter.paint(&content)?;
            fs::write(output, svg)?;
        }
        Commands::Mermaid { input, output } => {
            let content = fs::read_to_string(input)?;
            let painter = MermaidPainter;
            let svg = painter.paint(&content)?;
            fs::write(output, svg)?;
        }
        Commands::Server => {
            svgear::run_server().await?;
        }
    }

    Ok(())
}
