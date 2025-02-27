use svgear::{run_server, run_cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Determine mode of operation (CLI or server)
    if std::env::args().any(|arg| arg == "--server") {
        run_server().await?;
    } else {
        run_cli().await?;
    }
    Ok(())
}
