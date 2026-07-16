mod card;
mod display;
mod event_queue;
mod font;
mod graphics;
mod image;
mod image_observer;
mod input_method_handler;
mod jlet;
mod jlet_event_listener;
mod main;

pub use self::{
    card::Card, display::Display, event_queue::EventQueue, font::Font, graphics::Graphics, image::Image, image_observer::ImageObserver,
    input_method_handler::InputMethodHandler, jlet::Jlet, jlet_event_listener::JletEventListener, main::Main,
};

#[cfg(test)]
mod test {
    use java_constants::{ClassAccessFlags, MethodAccessFlags};
    use wie_jvm_support::WieJavaClassProto;

    use super::{Card, Display, EventQueue, Font, Graphics, Image, ImageObserver};

    fn assert_methods(proto: WieJavaClassProto, expected: &[(&str, &str, MethodAccessFlags)]) {
        for (name, descriptor, access_flags) in expected {
            let matching = proto
                .methods
                .iter()
                .filter(|method| method.name == *name && method.descriptor == *descriptor && method.access_flags == *access_flags)
                .count();
            assert_eq!(matching, 1, "{}.{name}{descriptor} with flags {access_flags:?}", proto.name);
        }
    }

    #[test]
    fn test_selected_lcdui_api_inventory() {
        assert_methods(
            Card::as_proto(),
            &[
                ("<init>", "(Z)V", MethodAccessFlags::empty()),
                ("<init>", "(IIII)V", MethodAccessFlags::empty()),
                ("<init>", "(Lorg/kwis/msp/lcdui/Display;IIII)V", MethodAccessFlags::empty()),
                ("<init>", "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V", MethodAccessFlags::empty()),
                ("move", "(II)V", MethodAccessFlags::empty()),
                ("resize", "(II)V", MethodAccessFlags::empty()),
                ("getX", "()I", MethodAccessFlags::empty()),
                ("getY", "()I", MethodAccessFlags::empty()),
                ("pointerNotify", "(III)Z", MethodAccessFlags::PROTECTED),
                ("getDisplay", "()Lorg/kwis/msp/lcdui/Display;", MethodAccessFlags::empty()),
            ],
        );
        assert_methods(
            Display::as_proto(),
            &[
                ("popCard", "()Lorg/kwis/msp/lcdui/Card;", MethodAccessFlags::empty()),
                ("removeCard", "(Lorg/kwis/msp/lcdui/Card;)Z", MethodAccessFlags::empty()),
                ("countCard", "()I", MethodAccessFlags::empty()),
                ("callSerially", "(Ljava/lang/Runnable;I)V", MethodAccessFlags::empty()),
                ("setDockedCard", "(Lorg/kwis/msp/lcdui/Card;I)V", MethodAccessFlags::empty()),
                ("isColor", "()Z", MethodAccessFlags::empty()),
                ("numColors", "()I", MethodAccessFlags::empty()),
                ("hasPointerEvents", "()Z", MethodAccessFlags::empty()),
                ("hasPointerMotionEvents", "()Z", MethodAccessFlags::empty()),
                ("hasRepeatEvents", "()Z", MethodAccessFlags::empty()),
                ("getKeyName", "(I)Ljava/lang/String;", MethodAccessFlags::STATIC),
                ("getBitsPerPixel", "()I", MethodAccessFlags::empty()),
                ("flush", "()V", MethodAccessFlags::empty()),
                (
                    "removeJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    MethodAccessFlags::empty(),
                ),
                ("grabKey", "(ILorg/kwis/msp/lcdui/JletEventListener;)V", MethodAccessFlags::empty()),
                ("ungrabKey", "(I)V", MethodAccessFlags::empty()),
            ],
        );
        assert_methods(
            EventQueue::as_proto(),
            &[
                ("postEvent", "([I)Z", MethodAccessFlags::empty()),
                ("postEvent", "(I[I)V", MethodAccessFlags::STATIC),
            ],
        );
        assert_methods(
            Font::as_proto(),
            &[
                ("getBaselinePosition", "()I", MethodAccessFlags::empty()),
                ("getFace", "()I", MethodAccessFlags::empty()),
                ("getSize", "()I", MethodAccessFlags::empty()),
                ("getStyle", "()I", MethodAccessFlags::empty()),
                ("isBold", "()Z", MethodAccessFlags::empty()),
                ("isItalic", "()Z", MethodAccessFlags::empty()),
                ("isPlain", "()Z", MethodAccessFlags::empty()),
                ("isUnderlined", "()Z", MethodAccessFlags::empty()),
            ],
        );
        assert_methods(
            Graphics::as_proto(),
            &[
                ("fillPolygon", "([I[I)V", MethodAccessFlags::empty()),
                ("getBlueComponent", "()I", MethodAccessFlags::empty()),
                ("getGrayScale", "()I", MethodAccessFlags::empty()),
                ("getGreenComponent", "()I", MethodAccessFlags::empty()),
                ("getRedComponent", "()I", MethodAccessFlags::empty()),
                ("getStrokeStyle", "()I", MethodAccessFlags::empty()),
                ("setStrokeStyle", "(I)V", MethodAccessFlags::empty()),
                ("getPixel", "(II)I", MethodAccessFlags::empty()),
                ("getPixels", "(IIII[BII)V", MethodAccessFlags::empty()),
                ("setPixels", "(IIII[BII)V", MethodAccessFlags::empty()),
                ("drawPolygon", "([I[I)V", MethodAccessFlags::empty()),
                ("reset", "()V", MethodAccessFlags::empty()),
                ("getAlpha", "()I", MethodAccessFlags::empty()),
                ("isXORMode", "()Z", MethodAccessFlags::empty()),
            ],
        );
        assert_methods(
            Image::as_proto(),
            &[
                ("<init>", "()V", MethodAccessFlags::PROTECTED),
                (
                    "loadImage",
                    "(Ljava/lang/String;Lorg/kwis/msp/lcdui/ImageObserver;)Lorg/kwis/msp/lcdui/Image;",
                    MethodAccessFlags::STATIC,
                ),
                ("isMutable", "()Z", MethodAccessFlags::empty()),
                ("isAnimated", "()Z", MethodAccessFlags::empty()),
                ("play", "(Lorg/kwis/msp/lcdui/ImageObserver;)V", MethodAccessFlags::empty()),
                ("stop", "()V", MethodAccessFlags::empty()),
                ("stopImage", "(Lorg/kwis/msp/lcdui/ImageObserver;)V", MethodAccessFlags::STATIC),
                ("drawImage", "(Lorg/kwis/msp/lcdui/Image;IIIIIIII)V", MethodAccessFlags::empty()),
                ("createSubImage", "(IIIIZ)Lorg/kwis/msp/lcdui/Image;", MethodAccessFlags::empty()),
                ("setTransparentColor", "(I)V", MethodAccessFlags::empty()),
            ],
        );

        let image_observer = ImageObserver::as_proto();
        assert_eq!(image_observer.access_flags, ClassAccessFlags::INTERFACE);
        assert_methods(
            image_observer,
            &[("notify", "(Lorg/kwis/msp/lcdui/Image;I)V", MethodAccessFlags::ABSTRACT)],
        );
    }
}
