use anyhow::Result;
use serde::{Deserialize, Serialize};

mod node_server;

pub use node_server::NodeServer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaintParams {
    pub ty: PaintType,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaintType {
    InlineTeX,
    Equation,
    Mermaid
}

/// A painter that can render different types of content to SVG
#[derive(Clone)]
pub struct Painter {
    node_server: Option<NodeServer>,
}

impl Painter {
    /// Create a new painter
    pub fn new() -> Self {
        Painter {
            node_server: None,
        }
    }

    /// Create a new painter with a Node server
    pub fn with_node_server(exe_path: String) -> Self {
        let mut painter = Self::new();
        painter.node_server = Some(NodeServer::new(exe_path));
        painter
    }

    /// Set the Node server
    pub fn set_node_server(&mut self, server: NodeServer) {
        self.node_server = Some(server);
    }

    /// Paint content to SVG
    pub async fn paint(&self, params: PaintParams) -> Result<String> {
        self.node_server.as_ref()
            .ok_or(anyhow::anyhow!("No Node server configured"))?
            .paint(params).await
    }
}
