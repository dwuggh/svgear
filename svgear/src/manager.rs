use anyhow::Result;
use fxhash::FxHashMap;
use resvg::{
    tiny_skia,
    usvg::{self, Tree},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, RwLock};

/// Represents a request to render an SVG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    /// SVG content to render
    pub svg_data: String,
    /// Desired width for rendering
    pub width: Option<u32>,
    /// Desired height for rendering
    pub height: Option<u32>,
    /// Optional ID to use instead of auto-generated hash
    pub id: Option<String>,
}

/// Response from rendering an SVG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResponse {
    /// ID of the rendered SVG (either provided or generated)
    pub id: String,
    /// Whether the SVG was newly rendered or retrieved from cache
    pub cached: bool,
    /// The bitmap data and dimensions
    pub bitmap: Bitmap,
}

/// Represents a request to retrieve a rendered bitmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBitmapRequest {
    /// ID of the SVG to retrieve
    pub id: String,
}

/// Bitmap structure to hold rendered image data and dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bitmap {
    /// The rendered bitmap as PNG bytes (base64 encoded when serialized)
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    /// Width of the bitmap
    pub width: u32,
    /// Height of the bitmap
    pub height: u32,
}

/// Response containing a rendered bitmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBitmapResponse {
    /// ID of the SVG
    pub id: String,
    /// The bitmap data and dimensions
    #[serde(flatten)]
    pub bitmap: Bitmap,
}

/// Manager for SVG storage and rendering
#[derive(Debug, Clone)]
pub struct SvgManager {
    /// Storage for original SVG data
    svgs: FxHashMap<String, String>,
    /// Storage for rendered bitmaps with their metadata
    bitmaps: FxHashMap<String, Bitmap>,
}

impl SvgManager {
    /// Create a new SVG manager
    pub fn new() -> Self {
        SvgManager {
            svgs: FxHashMap::default(),
            bitmaps: FxHashMap::default(),
        }
    }

    /// Generate a unique ID for an SVG
    pub fn generate_id(svg_data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(svg_data.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Store an SVG and return its ID
    pub fn store_svg(&mut self, svg_data: &str, custom_id: Option<String>) -> String {
        let id = custom_id.unwrap_or_else(|| Self::generate_id(svg_data));
        self.svgs.insert(id.clone(), svg_data.to_string());
        id
    }

    /// Get an SVG by ID
    pub fn get_svg(&self, id: &str) -> Option<&str> {
        self.svgs.get(id).map(|s| s.as_str())
    }

    /// Render an SVG to a bitmap with specified dimensions
    pub fn render_svg(
        &mut self,
        id: &str,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<(u32, u32)> {
        let svg_data = self
            .get_svg(id)
            .ok_or_else(|| anyhow::anyhow!("SVG not found"))?;

        // Parse the SVG
        let opt = usvg::Options::default();
        let tree = Tree::from_str(svg_data, &opt)?;

        // Get original size
        let orig_size = tree.size();

        // Calculate target size
        let (target_width, target_height) = match (width, height) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => {
                let aspect = orig_size.height() / orig_size.width();
                (w, (w as f32 * aspect) as u32)
            }
            (None, Some(h)) => {
                let aspect = orig_size.width() / orig_size.height();
                ((h as f32 * aspect) as u32, h)
            }
            (None, None) => (orig_size.width() as u32, orig_size.height() as u32),
        };

        // Create a pixmap with the target size
        let mut pixmap = tiny_skia::Pixmap::new(target_width, target_height)
            .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

        // Render the SVG
        resvg::render(&tree, usvg::Transform::identity(), &mut pixmap.as_mut());

        // Convert to PNG
        let png_data = pixmap.encode_png()?;

        // Store the bitmap with its metadata
        self.bitmaps.insert(
            id.to_string(),
            Bitmap {
                data: png_data,
                width: target_width,
                height: target_height,
            },
        );

        Ok((target_width, target_height))
    }

    /// Get a rendered bitmap by ID
    pub fn get_bitmap(&self, id: &str) -> Option<&Bitmap> {
        self.bitmaps.get(id)
    }

    /// Process a render request
    pub fn process_render_request(&mut self, request: RenderRequest) -> Result<RenderResponse> {
        // Generate or use provided ID
        let id = request
            .id
            .unwrap_or_else(|| Self::generate_id(&request.svg_data));

        // Check if we already have this SVG
        let cached = self.get_svg(&id).is_some();

        // Store the SVG if it's new
        if !cached {
            self.store_svg(&request.svg_data, Some(id.clone()));
        }

        // Render the SVG
        let (width, height) = self.render_svg(&id, request.width, request.height)?;

        // Get the bitmap
        let bitmap = self
            .get_bitmap(&id)
            .ok_or_else(|| anyhow::anyhow!("Bitmap not found after rendering"))?;

        Ok(RenderResponse {
            id,
            cached,
            bitmap: Bitmap {
                data: bitmap.data.clone(),
                width: bitmap.width,
                height: bitmap.height,
            },
        })
    }

    /// Process a get bitmap request
    pub fn process_get_bitmap_request(
        &self,
        request: GetBitmapRequest,
    ) -> Result<GetBitmapResponse> {
        let bitmap = self
            .get_bitmap(&request.id)
            .ok_or_else(|| anyhow::anyhow!("Bitmap not found"))?;

        Ok(GetBitmapResponse {
            id: request.id,
            bitmap: Bitmap {
                data: bitmap.data.clone(),
                width: bitmap.width,
                height: bitmap.height,
            },
        })
    }
}

/// Thread-safe wrapper around SvgManager
#[derive(Clone)]
pub struct SharedSvgManager(Arc<RwLock<SvgManager>>);

impl SharedSvgManager {
    /// Create a new shared SVG manager
    pub fn new() -> Self {
        SharedSvgManager(Arc::new(RwLock::new(SvgManager::new())))
    }

    /// Process a render request
    pub fn process_render_request(&self, request: RenderRequest) -> Result<RenderResponse> {
        self.0.write().unwrap().process_render_request(request)
    }

    /// Process a get bitmap request
    pub fn process_get_bitmap_request(
        &self,
        request: GetBitmapRequest,
    ) -> Result<GetBitmapResponse> {
        self.0.read().unwrap().process_get_bitmap_request(request)
    }

    /// Clone the shared manager
    pub fn clone(&self) -> Self {
        SharedSvgManager(Arc::clone(&self.0))
    }
}
