use crate::manager::{GetBitmapRequest, RenderRequest, SharedSvgManager};
use crate::painter::{Painter, PaintParams};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::{reply::json, reply::Json, reply::Reply, Filter};

/// RPC method types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Method {
    RenderSvg,
    GetBitmap,
    Paint,
    RenderToBitmap,
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

/// Result of a paint operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaintResult {
    pub svg: String,
}

/// Parameters for RenderToBitmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderToBitmapParams {
    pub paint_params: PaintParams,
    pub width: Option<u32>,
    pub height: Option<u32>,
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

/// Handle RenderSvg requests
async fn handle_render_svg(
    params: RenderRequest,
    server: &RpcServer,
    request_id: Option<String>,
) -> Json {
    match server.manager.process_render_request(params) {
        Ok(response) => json(&RpcResponse {
            result: Some(response),
            error: None,
            id: request_id,
        }),
        Err(e) => json(&RpcResponse::<()> {
            result: None,
            error: Some(format!("Error rendering SVG: {}", e)),
            id: request_id,
        }),
    }
}

/// Handle GetBitmap requests
async fn handle_get_bitmap(
    params: GetBitmapRequest,
    server: &RpcServer,
    request_id: Option<String>,
) -> Json {
    match server.manager.process_get_bitmap_request(params) {
        Ok(response) => json(&RpcResponse {
            result: Some(response),
            error: None,
            id: request_id,
        }),
        Err(e) => json(&RpcResponse::<()> {
            result: None,
            error: Some(format!("Error getting bitmap: {}", e)),
            id: request_id,
        }),
    }
}

/// Handle Paint requests
async fn handle_paint(
    params: PaintParams,
    server: &RpcServer,
    request_id: Option<String>,
) -> Json {
    match server.painter.paint(params).await {
        Ok(svg) => json(&RpcResponse {
            result: Some(PaintResult { svg }),
            error: None,
            id: request_id,
        }),
        Err(e) => json(&RpcResponse::<()> {
            result: None,
            error: Some(format!("Error painting: {}", e)),
            id: request_id,
        }),
    }
}

/// Handle RenderToBitmap requests
async fn handle_render_to_bitmap(
    params: RenderToBitmapParams,
    server: &RpcServer,
    request_id: Option<String>,
) -> Json {
    // Step 1: Paint to SVG
    let paint_result = match server.painter.paint(params.paint_params).await {
        Ok(svg) => svg,
        Err(e) => {
            return json(&RpcResponse::<()> {
                result: None,
                error: Some(format!("Error painting: {}", e)),
                id: request_id,
            })
        }
    };

    // Step 2: Render SVG to bitmap
    let render_request = RenderRequest {
        svg_data: paint_result,
        width: params.width,
        height: params.height,
        id: None,
    };

    match server.manager.process_render_request(render_request) {
        Ok(render_response) => {
            // Step 3: Get the bitmap
            let get_bitmap_request = GetBitmapRequest {
                id: render_response.id,
            };

            match server.manager.process_get_bitmap_request(get_bitmap_request) {
                Ok(bitmap_response) => json(&RpcResponse {
                    result: Some(bitmap_response),
                    error: None,
                    id: request_id,
                }),
                Err(e) => json(&RpcResponse::<()> {
                    result: None,
                    error: Some(format!("Error getting bitmap: {}", e)),
                    id: request_id,
                }),
            }
        }
        Err(e) => json(&RpcResponse::<()> {
            result: None,
            error: Some(format!("Error rendering SVG: {}", e)),
            id: request_id,
        }),
    }
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
        Some("RenderToBitmap") => Method::RenderToBitmap,
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

    // Get the request ID
    let request_id = request
        .get("id")
        .and_then(|id| id.as_str())
        .map(String::from);

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
                        id: request_id,
                    }));
                }
            };

            Ok(handle_render_svg(params, &server, request_id).await)
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
                        id: request_id,
                    }));
                }
            };

            Ok(handle_get_bitmap(params, &server, request_id).await)
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
                        id: request_id,
                    }));
                }
            };

            Ok(handle_paint(params, &server, request_id).await)
        }
        Method::RenderToBitmap => {
            let params: RenderToBitmapParams = match serde_json::from_value(
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
                        id: request_id,
                    }));
                }
            };

            Ok(handle_render_to_bitmap(params, &server, request_id).await)
        }
    }
}
