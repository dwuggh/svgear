use crate::manager::{GetBitmapRequest, RenderRequest, SharedSvgManager};
use crate::painter::{Painter, PaintParams};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::{reply::json, reply::Reply, Filter};

/// RPC method types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Method {
    RenderSvg,
    GetBitmap,
    Paint,
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
    painter: Painter,
}

impl Clone for RpcServer {
    fn clone(&self) -> Self {
        RpcServer {
            manager: self.manager.clone(),
            painter: self.painter.clone(),
        }
    }
}

impl RpcServer {
    /// Create a new RPC server
    pub fn new(manager: SharedSvgManager, painter: Painter) -> Self {
        RpcServer { manager, painter }
    }

    /// Start the RPC server
    pub async fn start(&self, port: u16) -> Result<()> {
        // Route for rendering SVGs
        let render_route = warp::path("rpc")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_manager(self.clone()))
            .and_then(handle_rpc);


        println!("Starting RPC server on port {}", port);
        warp::serve(render_route).run(([127, 0, 0, 1], port)).await;

        Ok(())
    }
}

/// Helper to inject the manager into route handlers
fn with_manager(
    server: RpcServer,
) -> impl Filter<Extract = (RpcServer,), Error = Infallible> + Clone {
    warp::any().map(move || server.clone())
}

/// Handle RPC requests
async fn handle_rpc(
    request: serde_json::Value,
    server: RpcServer,
) -> Result<impl Reply, Infallible> {
    // Parse the method
    let method = match request.get("method").and_then(|m| m.as_str()) {
        Some("RenderSvg") => Method::RenderSvg,
        Some("GetBitmap") => Method::GetBitmap,
        Some("Paint") => Method::Paint,
        _ => {
            return Ok(json(&RpcResponse::<()> {
                result: None,
                error: Some("Unknown method".to_string()),
                id: request
                    .get("id")
                    .and_then(|id| id.as_str())
                    .map(String::from),
            }));
        }
    };

    // Process based on method
    match method {
        Method::RenderSvg => {
            let params: RenderRequest = match serde_json::from_value(
                request
                    .get("params")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            ) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(json(&RpcResponse::<()> {
                        result: None,
                        error: Some(format!("Invalid parameters: {}", e)),
                        id: request
                            .get("id")
                            .and_then(|id| id.as_str())
                            .map(String::from),
                    }));
                }
            };

            match server.manager.process_render_request(params) {
                Ok(response) => Ok(json(&RpcResponse {
                    result: Some(response),
                    error: None,
                    id: request
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(String::from),
                })),
                Err(e) => Ok(json(&RpcResponse::<()> {
                    result: None,
                    error: Some(format!("Error rendering SVG: {}", e)),
                    id: request
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(String::from),
                })),
            }
        }
        Method::GetBitmap => {
            let params: GetBitmapRequest = match serde_json::from_value(
                request
                    .get("params")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            ) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(json(&RpcResponse::<()> {
                        result: None,
                        error: Some(format!("Invalid parameters: {}", e)),
                        id: request
                            .get("id")
                            .and_then(|id| id.as_str())
                            .map(String::from),
                    }));
                }
            };

            match server.manager.process_get_bitmap_request(params) {
                Ok(response) => Ok(json(&RpcResponse {
                    result: Some(response),
                    error: None,
                    id: request
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(String::from),
                })),
                Err(e) => Ok(json(&RpcResponse::<()> {
                    result: None,
                    error: Some(format!("Error getting bitmap: {}", e)),
                    id: request
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(String::from),
                })),
            }
        }
        Method::Paint => {
            let params: PaintParams = match serde_json::from_value(
                request
                    .get("params")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            ) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(json(&RpcResponse::<()> {
                        result: None,
                        error: Some(format!("Invalid parameters: {}", e)),
                        id: request
                            .get("id")
                            .and_then(|id| id.as_str())
                            .map(String::from),
                    }));
                }
            };

            // Create a response type for Paint
            #[derive(Serialize)]
            struct PaintResult {
                svg: String,
            }

            // Call the painter
            match server.painter.paint(params).await {
                Ok(svg) => Ok(json(&RpcResponse {
                    result: Some(PaintResult { svg }),
                    error: None,
                    id: request
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(String::from),
                })),
                Err(e) => Ok(json(&RpcResponse::<()> {
                    result: None,
                    error: Some(format!("Error painting: {}", e)),
                    id: request
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(String::from),
                })),
            }
        }
    }
}
