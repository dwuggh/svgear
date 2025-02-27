use super::Painter;
use anyhow::Result;

pub struct Mermaid;

impl Painter for Mermaid {
    fn paint(&self, content: &str) -> Result<String> {
        // Send request to Mermaid server and return SVG
        Ok(String::new())
    }
}
