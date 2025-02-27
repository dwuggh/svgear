use anyhow::Result;

mod mermaid;
mod mathjax_server;

pub use mermaid::Mermaid;
pub use mathjax_server::MathjaxServer;

pub trait Painter {
    fn paint(&self, content: &str) -> Result<String>;
}
