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
pub use painter::Painter;
pub use rpc::{Method, RpcRequest, RpcResponse, RpcServer};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CmdType {
    TeX,
    Mermaid,
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
