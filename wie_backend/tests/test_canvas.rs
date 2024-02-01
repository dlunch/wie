use wie_backend::canvas::{ArgbPixel, Canvas, Color, Image, ImageBuffer, VecImageBuffer};

#[test]
fn test_canvas() -> anyhow::Result<()> {
    let mut image_buffer = VecImageBuffer::<ArgbPixel>::new(10, 10);
    let mut canvas = Canvas::new(&mut image_buffer as &mut dyn ImageBuffer);

    canvas.fill_rect(0, 0, 10, 10, Color { r: 0, g: 0, b: 0, a: 255 });

    let raw = image_buffer.raw();

    assert_eq!(raw.len(), 10 * 10 * 4);
    for i in 0..10 * 10 {
        assert_eq!(raw[i * 4], 0);
        assert_eq!(raw[i * 4 + 1], 0);
        assert_eq!(raw[i * 4 + 2], 0);
        assert_eq!(raw[i * 4 + 3], 255);
    }

    Ok(())
}
