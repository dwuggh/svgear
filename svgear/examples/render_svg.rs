use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use svgear::{RpcServer, SharedSvgManager, SvgClient};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// SVG file to render
    #[arg(short, long)]
    input: PathBuf,

    /// Output PNG file
    #[arg(short, long)]
    output: PathBuf,

    /// Width of the output image
    #[arg(short, long)]
    width: Option<u32>,

    /// Height of the output image
    #[arg(short, long)]
    height: Option<u32>,

    /// Run in server mode
    #[arg(long)]
    server: bool,

    /// Port for server mode
    #[arg(long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.server {
        // Start the server
        println!("Starting SVG rendering server on port {}", cli.port);
        let manager = SharedSvgManager::new();
        let server = RpcServer::new(manager);
        server.start(cli.port).await?;
    } else {
        // Use the client to render an SVG
        let svg_data = std::fs::read_to_string(&cli.input)?;
        let client = SvgClient::new("localhost", 3000);

        println!("Rendering SVG: {}", cli.input.display());
        let response = client.render_svg(&svg_data, cli.width, cli.height).await?;

        println!("Saving bitmap to: {}", cli.output.display());
        client
            .save_bitmap(&response.id, cli.output.to_str().unwrap())
            .await?;

        println!("Done! Image size: {}x{}", response.width, response.height);
    }

    Ok(())
}
