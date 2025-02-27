use anyhow::Result;

use super::Painter;

pub struct MathjaxServer;

impl Painter for MathjaxServer {
    fn paint(&self, content: &str) -> Result<String> {
        // Send request to MathJax server and return SVG
        Ok(String::new())
    }
}
