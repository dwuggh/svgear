use crate::manager::{
    SharedSvgManager, RenderRequest, RenderResponse, 
    GetBitmapRequest, GetBitmapResponse
};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use warp::{Filter, reply::json, reply::Reply};
use std::convert::Infallible;

/// RPC method types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Method {
    RenderSvg,
    GetBitmap,
}

/// Generic RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest<T> {
    pub method: Method,
    pub params: T,
    pub id: Option<String>,
}

/// Generic RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub result: Option<T>,
    pub error: Option<String>,
    pub id: Option<String>,
}

/// RPC server for SVG rendering
pub struct RpcServer {
    manager: SharedSvgManager,
}

impl RpcServer {
    /// Create a new RPC server
    pub fn new(manager: SharedSvgManager) -> Self {
        RpcServer { manager }
    }
    
    /// Start the RPC server
    pub async fn start(&self, port: u16) -> Result<()> {
        let manager = self.manager.clone();
        
        // Route for rendering SVGs
        let render_route = warp::path("rpc")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_manager(manager.clone()))
            .and_then(handle_rpc);
            
        println!("Starting RPC server on port {}", port);
        warp::serve(render_route)
            .run(([127, 0, 0, 1], port))
            .await;
            
        Ok(())
    }
}

/// Helper to inject the manager into route handlers
fn with_manager(manager: SharedSvgManager) -> impl Filter<Extract = (SharedSvgManager,), Error = Infallible> + Clone {
    warp::any().map(move || manager.clone())
}

/// Handle RPC requests
async fn handle_rpc(
    request: serde_json::Value,
    manager: SharedSvgManager,
) -> Result<impl Reply, Infallible> {
    // Parse the method
    let method = match request.get("method").and_then(|m| m.as_str()) {
        Some("RenderSvg") => Method::RenderSvg,
        Some("GetBitmap") => Method::GetBitmap,
        _ => {
            return Ok(json(&RpcResponse::<()> {
                result: None,
                error: Some("Unknown method".to_string()),
                id: request.get("id").and_then(|id| id.as_str()).map(String::from),
            }));
        }
    };
    
    // Process based on method
    match method {
        Method::RenderSvg => {
            let params: RenderRequest = match serde_json::from_value(
                request.get("params").cloned().unwrap_or(serde_json::Value::Null)
            ) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(json(&RpcResponse::<()> {
                        result: None,
                        error: Some(format!("Invalid parameters: {}", e)),
                        id: request.get("id").and_then(|id| id.as_str()).map(String::from),
                    }));
                }
            };
            
            match manager.process_render_request(params) {
                Ok(response) => {
                    Ok(json(&RpcResponse {
                        result: Some(response),
                        error: None,
                        id: request.get("id").and_then(|id| id.as_str()).map(String::from),
                    }))
                },
                Err(e) => {
                    Ok(json(&RpcResponse::<()> {
                        result: None,
                        error: Some(format!("Error rendering SVG: {}", e)),
                        id: request.get("id").and_then(|id| id.as_str()).map(String::from),
                    }))
                }
            }
        },
        Method::GetBitmap => {
            let params: GetBitmapRequest = match serde_json::from_value(
                request.get("params").cloned().unwrap_or(serde_json::Value::Null)
            ) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(json(&RpcResponse::<()> {
                        result: None,
                        error: Some(format!("Invalid parameters: {}", e)),
                        id: request.get("id").and_then(|id| id.as_str()).map(String::from),
                    }));
                }
            };
            
            match manager.process_get_bitmap_request(params) {
                Ok(response) => {
                    Ok(json(&RpcResponse {
                        result: Some(response),
                        error: None,
                        id: request.get("id").and_then(|id| id.as_str()).map(String::from),
                    }))
                },
                Err(e) => {
                    Ok(json(&RpcResponse::<()> {
                        result: None,
                        error: Some(format!("Error getting bitmap: {}", e)),
                        id: request.get("id").and_then(|id| id.as_str()).map(String::from),
                    }))
                }
            }
        }
    }
}
