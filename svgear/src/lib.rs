pub mod painter;
mod http_painter;
mod mermaid_painter;

pub use http_painter::HttpPainter;
pub use mermaid_painter::MermaidPainter;
pub use painter::Painter;

pub async fn run_server() -> anyhow::Result<()> {
    // Implement server logic here
    Ok(())
}

pub async fn run_cli() -> anyhow::Result<()> {
    // Implement CLI logic here
    Ok(())
}
