use anyhow::Result;
use svgear::SvgManager;

#[test]
fn test_svg_manager() -> Result<()> {
    // Simple SVG for testing
    let svg_data = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <rect width="100" height="100" fill="red" />
    </svg>"#;

    // Create a manager
    let mut manager = SvgManager::new();

    // Store the SVG
    let id = manager.store_svg(svg_data, None);

    // Verify we can retrieve it
    let retrieved = manager.get_svg(&id).unwrap();
    assert_eq!(retrieved, svg_data);

    // Render it
    let (width, height) = manager.render_svg(&id, Some(200), None)?;

    // Verify dimensions
    assert_eq!(width, 200);
    assert_eq!(height, 200); // Should maintain aspect ratio (1:1)

    // Get the bitmap
    let (bitmap, w, h) = manager.get_bitmap(&id).unwrap();

    // Verify bitmap exists and has correct dimensions
    assert!(!bitmap.is_empty());
    assert_eq!(w, 200);
    assert_eq!(h, 200);

    Ok(())
}
