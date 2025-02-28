pub mod client;
pub mod error;
pub mod manager;
pub mod painter;
pub mod rpc;

pub use client::SvgClient;
pub use manager::{
    GetBitmapRequest, GetBitmapResponse, RenderRequest, RenderResponse, SharedSvgManager,
    SvgManager,
};
pub use painter::{Painter, PaintParams, PaintType};
pub use rpc::{Method, PaintResult, RenderToBitmapParams, RpcRequest, RpcResponse, RpcServer};

pub async fn run_server(port: u16, exe_path: String) -> anyhow::Result<()> {
    let manager = SharedSvgManager::new();
    let painter = Painter::with_node_server(exe_path);
    let server = RpcServer::new(manager, painter);
    server.start(port).await
}

pub async fn run_cli() -> anyhow::Result<()> {
    // Implement CLI logic here
    Ok(())
}







