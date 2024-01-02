use wie_backend::canvas::{ArgbPixel, Canvas, Color, Image, ImageBuffer};

#[test]
fn test_canvas() -> anyhow::Result<()> {
    let mut canvas = ImageBuffer::<ArgbPixel>::new(10, 10);

    canvas.fill_rect(0, 0, 10, 10, Color { r: 0, g: 0, b: 0, a: 255 });

    let raw = canvas.raw();

    assert_eq!(raw.len(), 10 * 10 * 4);
    for i in 0..10 * 10 {
        assert_eq!(raw[i * 4], 0);
        assert_eq!(raw[i * 4 + 1], 0);
        assert_eq!(raw[i * 4 + 2], 0);
        assert_eq!(raw[i * 4 + 3], 255);
    }

    Ok(())
}
