use crate::manager::{GetBitmapRequest, GetBitmapResponse, RenderRequest, RenderResponse};
use crate::rpc::{Method, RpcRequest, RpcResponse};
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;

/// Client for the SVG rendering RPC server
pub struct SvgClient {
    client: Client,
    base_url: String,
}

impl SvgClient {
    /// Create a new SVG client
    pub fn new(host: &str, port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        SvgClient {
            client,
            base_url: format!("http://{}:{}/rpc", host, port),
        }
    }

    /// Send an RPC request
    async fn send_request<T, R>(&self, method: Method, params: T) -> Result<R>
    where
        T: serde::Serialize,
        R: DeserializeOwned,
    {
        let request_id = uuid::Uuid::new_v4().to_string();

        let request = RpcRequest {
            method,
            params,
            id: Some(request_id.clone()),
        };

        let response = self
            .client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }

        let rpc_response: RpcResponse<R> = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(anyhow!("RPC error: {}", error));
        }

        rpc_response
            .result
            .ok_or_else(|| anyhow!("No result in response"))
    }

    /// Render an SVG
    pub async fn render_svg(
        &self,
        svg_data: &str,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<RenderResponse> {
        let request = RenderRequest {
            svg_data: svg_data.to_string(),
            width,
            height,
            id: None,
        };

        self.send_request(Method::RenderSvg, request).await
    }

    /// Get a bitmap by ID
    pub async fn get_bitmap(&self, id: &str) -> Result<GetBitmapResponse> {
        let request = GetBitmapRequest { id: id.to_string() };

        self.send_request(Method::GetBitmap, request).await
    }

    /// Save a bitmap to a file
    pub async fn save_bitmap(&self, id: &str, path: &str) -> Result<()> {
        let response = self.get_bitmap(id).await?;
        std::fs::write(path, &response.bitmap)?;
        Ok(())
    }
}
