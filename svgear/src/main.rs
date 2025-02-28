use std::io::Write;

use anyhow::Result;
use clap::{Parser, Subcommand};
use svgear::painter::{MathjaxServer, Mermaid};
use svgear::{PaintType, Painter, RenderRequest, SharedSvgManager};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate SVG from MathJax
    Render {
        /// input content
        input: String,
        #[arg(short, long)]
        input_type: String,
        #[arg(short, long)]
        output_type: String,
        #[arg(short, long)]
        width: Option<u32>,
        #[arg(short, long)]
        height: Option<u32>,
        #[arg(short, long)]
        output: String,
    },
    /// Run in server mode
    Serve,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render { input, input_type, output_type, width, height, output } => {
            match input_type.as_str() {
                "svg" => {
                    let mut manager = svgear::SvgManager::new();
                    let resp = manager.process_render_request(RenderRequest {
                        svg_data: input,
                        width,
                        height,
                        id: None,
                    })?;
                    use std::io::Write;
                    let mut stdout = std::io::stdout();
                    stdout.write(&resp.bitmap.data)?;
                }
                _ => (),
            }
            todo!()
        }
        Commands::Serve => {
            svgear::run_server(3000).await?;
        }
    }

    Ok(())
}






