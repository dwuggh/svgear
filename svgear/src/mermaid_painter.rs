use super::painter::Painter;
use anyhow::Result;

pub struct MermaidPainter;

impl Painter for MermaidPainter {
    fn paint(&self, content: &str) -> Result<String> {
        // Send request to Mermaid server and return SVG
        Ok(String::new())
    }
}
