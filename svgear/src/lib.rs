pub mod manager;
pub mod painter;
pub mod http_painter;
pub mod mermaid_painter;
pub mod error;

use manager::SvgManager;
use painter::Painter;

pub async fn run_server() -> anyhow::Result<()> {
    // Implement server logic here
    Ok(())
}

pub async fn run_cli() -> anyhow::Result<()> {
    // Implement CLI logic here
    Ok(())
}
