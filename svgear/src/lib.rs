pub mod painter;

pub use painter::Painter;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum CmdType {
    TeX,
    Mermaid
}

pub async fn run_server() -> anyhow::Result<()> {
    // Implement server logic here
    Ok(())
}

pub async fn run_cli() -> anyhow::Result<()> {
    // Implement CLI logic here
    Ok(())
}
