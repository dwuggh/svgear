use super::Painter;
use anyhow::Result;

pub struct Mermaid;

#[async_trait::async_trait]
impl Painter for Mermaid {
    async fn paint(&self, content: &str) -> Result<String> {
        // Send request to Mermaid server and return SVG
        Ok(String::new())
    }
}
