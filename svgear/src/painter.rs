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

#[async_trait::async_trait]
pub trait Painter {
    async fn paint(&self, content: &str) -> Result<String>;
}
