use super::painter::Painter;
use anyhow::Result;

pub struct HttpPainter;

impl Painter for HttpPainter {
    fn paint(&self, content: &str) -> Result<String> {
        // Send request to MathJax server and return SVG
        Ok(String::new())
    }
}
