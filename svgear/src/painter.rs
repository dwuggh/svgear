use anyhow::Result;
use serde::{Deserialize, Serialize};

mod mathjax_server;
mod mermaid;

pub use mathjax_server::MathjaxServer;

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
    mathjax: Option<MathjaxServer>,
}

impl Painter {
    /// Create a new painter
    pub fn new() -> Self {
        Painter {
            mathjax: None,
        }
    }

    /// Create a new painter with a MathJax server
    pub fn with_mathjax(exe_path: String) -> Self {
        let mut painter = Self::new();
        painter.mathjax = Some(MathjaxServer::new(exe_path));
        painter
    }

    /// Set the MathJax server
    pub fn set_mathjax(&mut self, server: MathjaxServer) {
        self.mathjax = Some(server);
    }

    /// Paint content to SVG
    pub async fn paint(&self, params: PaintParams) -> Result<String> {
        self.mathjax.ok_or(anyhow::anyhow!("no node server"))?.paint(params).await
    }
}













