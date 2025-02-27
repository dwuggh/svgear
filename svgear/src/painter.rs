use anyhow::Result;

mod mathjax_server;
mod mermaid;

pub use mathjax_server::MathjaxServer;
pub use mermaid::Mermaid;

pub trait Painter {
    fn paint(&self, content: &str) -> Result<String>;
}
