pub mod painter;
pub mod manager;
pub mod rpc;
pub mod error;
pub mod client;

pub use painter::Painter;
pub use manager::{
    SvgManager, SharedSvgManager, 
    RenderRequest, RenderResponse,
    GetBitmapRequest, GetBitmapResponse
};
pub use rpc::{RpcServer, Method, RpcRequest, RpcResponse};
pub use client::SvgClient;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CmdType {
    TeX,
    Mermaid
}

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    let manager = SharedSvgManager::new();
    let server = RpcServer::new(manager);
    server.start(port).await
}

pub async fn run_cli() -> anyhow::Result<()> {
    // Implement CLI logic here
    Ok(())
}
