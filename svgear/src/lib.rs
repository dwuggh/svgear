pub mod client;
pub mod error;
pub mod manager;
pub mod painter;
pub mod rpc;

use std::sync::Arc;

pub use client::SvgClient;
pub use manager::{
    GetBitmapRequest, GetBitmapResponse, RenderRequest, RenderResponse, SharedSvgManager,
    SvgManager,
};
pub use painter::{PaintParams, PaintType, Painter};
pub use rpc::{Method, PaintResult, RenderToBitmapParams, RpcRequest, RpcResponse, RpcServer};
use tokio::{
    runtime::{Builder, Runtime},
    sync::RwLock,
};
pub use tokio;

#[derive(Debug)]
pub struct Svgear {
    pub manager: SvgManager,
    pub painter: Painter,
}


impl Svgear {
    pub fn new(exe_path: String) -> Self {
        Svgear {
            manager: SvgManager::new(),
            painter: Painter::with_node_server(exe_path),
        }
    }
}

