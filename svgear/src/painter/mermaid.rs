use anyhow::Result;

use super::{PaintParams, PaintType};

pub struct Mermaid;

impl Mermaid {
    /// Create a new Mermaid renderer
    pub fn new() -> Self {
        Mermaid
    }
    
    /// Paint Mermaid diagram to SVG
    pub async fn paint(&self, params: PaintParams) -> Result<String> {
        // Verify this is a Mermaid diagram
        if params.ty != PaintType::Mermaid {
            return Err(anyhow::anyhow!("Unsupported paint type for Mermaid: {:?}", params.ty));
        }
        
        // Send request to Mermaid server and return SVG
        // TODO: Implement actual Mermaid rendering
        Ok(String::new())
    }
}
