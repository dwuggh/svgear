use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::SvgearError;
use super::{PaintParams, PaintType};

/// Request to the MathJax server
#[derive(Debug, Serialize, Deserialize)]
struct MathJaxRequest {
    /// Whether the equation is inline or display mode
    inline: bool,
    /// The equation content to render
    content: String,
}

/// Configuration for the MathJax server
#[derive(Debug, Clone)]
pub struct MathjaxServer {
    /// Server address (hostname or IP)
    address: String,
    /// Server port
    port: u16,
    /// HTTP client for making requests
    client: Client,
}

impl MathjaxServer {
    /// Create a new MathJax server connection
    pub fn new(address: String, port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        MathjaxServer {
            address,
            port,
            client,
        }
    }

    /// Get the server URL
    fn server_url(&self) -> String {
        format!("http://{}:{}/convert", self.address, self.port)
    }
    
    /// Paint TeX content to SVG
    pub async fn paint(&self, params: PaintParams) -> Result<String> {
        // Determine if the content is inline or display mode based on the paint type
        let inline = match params.ty {
            PaintType::InlineTeX => true,
            PaintType::Equation => false,
            _ => return Err(anyhow::anyhow!("Unsupported paint type for MathJax: {:?}", params.ty)),
        };

        // Create the request
        let request = MathJaxRequest {
            inline,
            content: params.content,
        };

        // Send the request to the MathJax server
        let response = self.client
            .post(&self.server_url())
            .json(&request)
            .send()
            .await?;

        // Check if the request was successful
        if !response.status().is_success() {
            return Err(SvgearError::HttpError(response.error_for_status().unwrap_err()).into());
        }

        // Parse the response as text (SVG content)
        let svg_content = response.text().await?;
        
        // Return the SVG content
        Ok(svg_content)
    }
}
