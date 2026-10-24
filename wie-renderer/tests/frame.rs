use wie_backend::canvas::{AbgrPixel, ArgbPixel, Image, VecImageBuffer};
use wie_renderer::Frame;

#[test]
fn converts_guest_colors_to_opaque_rgba_without_reinterpreting_storage() {
    let argb = VecImageBuffer::<ArgbPixel>::from_raw(3, 1, vec![0x00123456, 0x80ff0000, 0xff0000ff]);
    let abgr = VecImageBuffer::<AbgrPixel>::from_raw(3, 1, vec![0x00563412, 0x800000ff, 0xffff0000]);
    for image in [&argb as &dyn Image, &abgr] {
        let frame = Frame::from_image(image);
        assert_eq!((frame.width, frame.height), (3, 1));
        assert_eq!(frame.rgba, [0x12, 0x34, 0x56, 255, 255, 0, 0, 255, 0, 0, 255, 255]);
    }
}
