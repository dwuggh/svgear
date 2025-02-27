use fxhash::FxHashMap;
use resvg::usvg::Tree;
use anyhow::Result;

pub struct SvgManager {
    bitmaps: FxHashMap<String, Vec<u8>>,
}

impl SvgManager {
    pub fn new() -> Self {
        SvgManager {
            bitmaps: FxHashMap::default(),
        }
    }

    pub fn scale_svg(&mut self, id: &str, svg_ &str, width: u32, height: u32) -> Result<()> {
        let tree = Tree::from_str(svg_data, &resvg::usvg::Options::default())?;
        // Perform scaling and store bitmap
        Ok(())
    }
}
