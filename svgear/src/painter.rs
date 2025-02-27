use anyhow::Result;
use serde::{Deserialize, Serialize};

mod mathjax_server;
mod mermaid;

pub use mathjax_server::MathjaxServer;
pub use mermaid::Mermaid;

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
    mermaid: Option<Mermaid>,
}

impl Painter {
    /// Create a new painter
    pub fn new() -> Self {
        Painter {
            mathjax: None,
            mermaid: None,
        }
    }

    /// Create a new painter with a MathJax server
    pub fn with_mathjax(address: String, port: u16) -> Self {
        let mut painter = Self::new();
        painter.mathjax = Some(MathjaxServer::new(address, port));
        painter
    }

    /// Create a new painter with a Mermaid renderer
    pub fn with_mermaid() -> Self {
        let mut painter = Self::new();
        painter.mermaid = Some(Mermaid::new());
        painter
    }

    /// Set the MathJax server
    pub fn set_mathjax(&mut self, server: MathjaxServer) {
        self.mathjax = Some(server);
    }

    /// Set the Mermaid renderer
    pub fn set_mermaid(&mut self, mermaid: Mermaid) {
        self.mermaid = Some(mermaid);
    }

    /// Paint content to SVG
    pub async fn paint(&self, params: PaintParams) -> Result<String> {
        match params.ty {
            PaintType::InlineTeX | PaintType::Equation => {
                let mathjax = self.mathjax.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("MathJax server not configured"))?;
                mathjax.paint(params).await
            },
            PaintType::Mermaid => {
                let mermaid = self.mermaid.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Mermaid renderer not configured"))?;
                mermaid.paint(params).await
            }
        }
    }
}
