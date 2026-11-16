use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_backend::canvas::Image as BackendImage;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Graphics, Image, game::TiledLayer};

const CLASS: &str = "javax/microedition/lcdui/game/Sprite";
const LAYER: &str = "javax/microedition/lcdui/game/Layer";

// Half-open rectangle in the painter's coordinate system.
#[derive(Clone, Copy)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            left: x,
            top: y,
            right: x.saturating_add(width),
            bottom: y.saturating_add(height),
        }
    }

    fn intersect(&self, other: &Rect) -> Rect {
        Rect {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }

    fn is_empty(&self) -> bool {
        self.right <= self.left || self.bottom <= self.top
    }

    fn points(self) -> impl Iterator<Item = (i32, i32)> {
        (self.top..self.bottom).flat_map(move |y| (self.left..self.right).map(move |x| (x, y)))
    }
}

// Where a sprite's current frame sits on screen and in its source image.
struct Placement {
    x: i32,
    y: i32,
    image: ClassInstanceRef<Image>,
    frame_x: i32,
    frame_y: i32,
    frame_width: i32,
    frame_height: i32,
    transform: i32,
    // Collision rectangle relative to the untransformed frame: x, y, width, height.
    collision: [i32; 4],
}

impl Placement {
    fn bounds(&self) -> Rect {
        let (width, height) = transformed_size(self.transform, self.frame_width, self.frame_height);
        Rect::new(self.x, self.y, width, height)
    }

    fn collision_bounds(&self) -> Rect {
        let [x, y, width, height] = self.collision;
        if width == 0 || height == 0 {
            return Rect::new(self.x, self.y, 0, 0);
        }

        let first = transform_point(self.transform, self.frame_width, self.frame_height, x, y);
        let last = transform_point(self.transform, self.frame_width, self.frame_height, x + width - 1, y + height - 1);
        Rect {
            left: self.x + first.0.min(last.0),
            top: self.y + first.1.min(last.1),
            right: self.x + first.0.max(last.0) + 1,
            bottom: self.y + first.1.max(last.1) + 1,
        }
    }

    // `x` and `y` must lie within `bounds()`.
    fn is_opaque(&self, pixels: &dyn BackendImage, x: i32, y: i32) -> bool {
        let (source_x, source_y) = Image::transform_source(self.transform, self.frame_width, self.frame_height, x - self.x, y - self.y);

        pixels.get_pixel(self.frame_x + source_x, self.frame_y + source_y).a == 0xff
    }
}

// Transforms 4 to 7 include a quarter turn, which swaps the dimensions.
fn transformed_size(transform: i32, width: i32, height: i32) -> (i32, i32) {
    if transform >= 4 { (height, width) } else { (width, height) }
}

// Maps a point of the untransformed width x height frame to its position in the transformed frame.
fn transform_point(transform: i32, width: i32, height: i32, x: i32, y: i32) -> (i32, i32) {
    match transform {
        1 => (x, height - 1 - y),             // TRANS_MIRROR_ROT180
        2 => (width - 1 - x, y),              // TRANS_MIRROR
        3 => (width - 1 - x, height - 1 - y), // TRANS_ROT180
        4 => (y, x),                          // TRANS_MIRROR_ROT270
        5 => (height - 1 - y, x),             // TRANS_ROT90
        6 => (y, width - 1 - x),              // TRANS_ROT270
        7 => (height - 1 - y, width - 1 - x), // TRANS_MIRROR_ROT90
        _ => (x, y),                          // TRANS_NONE
    }
}

// class javax.microedition.lcdui.game.Sprite
pub struct Sprite;

impl Sprite {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/game/Sprite",
            parent_class: Some("javax/microedition/lcdui/game/Layer"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Image;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/lcdui/Image;II)V",
                    Self::init_with_frames,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/lcdui/game/Sprite;)V",
                    Self::init_with_sprite,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("defineReferencePixel", "(II)V", Self::define_reference_pixel, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setRefPixelPosition", "(II)V", Self::set_ref_pixel_position, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRefPixelX", "()I", Self::get_ref_pixel_x, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRefPixelY", "()I", Self::get_ref_pixel_y, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFrame", "(I)V", Self::set_frame, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getFrame", "()I", Self::get_frame, MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL),
                JavaMethodProto::new("getRawFrameCount", "()I", Self::get_raw_frame_count, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getFrameSequenceLength",
                    "()I",
                    Self::get_frame_sequence_length,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("nextFrame", "()V", Self::next_frame, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("prevFrame", "()V", Self::prev_frame, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::paint,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new("setFrameSequence", "([I)V", Self::set_frame_sequence, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setImage",
                    "(Ljavax/microedition/lcdui/Image;II)V",
                    Self::set_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "defineCollisionRectangle",
                    "(IIII)V",
                    Self::define_collision_rectangle,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setTransform", "(I)V", Self::set_transform, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/game/Sprite;Z)Z",
                    Self::collides_with_sprite,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new(
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/game/TiledLayer;Z)Z",
                    Self::collides_with_tiled_layer,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
                JavaMethodProto::new(
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/Image;IIZ)Z",
                    Self::collides_with_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::FINAL,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("image", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("frameWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("frameHeight", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("rawFrameCount", "I", FieldAccessFlags::PRIVATE),
                // Null while the default sequence of all raw frames in order is in use.
                JavaFieldProto::new("frameSequence", "[I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("frame", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("transform", "I", FieldAccessFlags::PRIVATE),
                // The reference pixel and collision rectangle are relative to the untransformed frame.
                JavaFieldProto::new("refX", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("refY", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("collisionX", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("collisionY", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("collisionWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("collisionHeight", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::<init>({this:?}, {image:?})");

        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        let width: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
            .await?;

        jvm.invoke_special(&this, CLASS, "<init>", "(Ljavax/microedition/lcdui/Image;II)V", (image, width, height))
            .await
    }

    async fn init_with_frames(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        frame_width: i32,
        frame_height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::<init>({this:?}, {image:?}, {frame_width}, {frame_height})");

        let _: () = jvm.invoke_special(&this, LAYER, "<init>", "()V", ()).await?;

        jvm.invoke_virtual(
            &this,
            CLASS,
            "setImage",
            "(Ljavax/microedition/lcdui/Image;II)V",
            (image, frame_width, frame_height),
        )
        .await
    }

    async fn init_with_sprite(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, sprite: ClassInstanceRef<Sprite>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::<init>({this:?}, {sprite:?})");

        if sprite.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "sprite is null").await);
        }
        let _: () = jvm.invoke_special(&this, LAYER, "<init>", "()V", ()).await?;

        let image: ClassInstanceRef<Image> = jvm.get_field(&sprite, CLASS, "image", "Ljavax/microedition/lcdui/Image;").await?;
        jvm.put_field(&mut this, CLASS, "image", "Ljavax/microedition/lcdui/Image;", image)
            .await?;
        // The sequence array is replaced, never modified in place, so both sprites can refer to the same one.
        let frame_sequence: ClassInstanceRef<Array<i32>> = jvm.get_field(&sprite, CLASS, "frameSequence", "[I").await?;
        jvm.put_field(&mut this, CLASS, "frameSequence", "[I", frame_sequence).await?;
        for (class, field) in [
            (CLASS, "frameWidth"),
            (CLASS, "frameHeight"),
            (CLASS, "rawFrameCount"),
            (CLASS, "frame"),
            (CLASS, "transform"),
            (CLASS, "refX"),
            (CLASS, "refY"),
            (CLASS, "collisionX"),
            (CLASS, "collisionY"),
            (CLASS, "collisionWidth"),
            (CLASS, "collisionHeight"),
            (LAYER, "x"),
            (LAYER, "y"),
            (LAYER, "width"),
            (LAYER, "height"),
        ] {
            let value: i32 = jvm.get_field(&sprite, class, field, "I").await?;
            jvm.put_field(&mut this, class, field, "I", value).await?;
        }
        let visible: bool = jvm.get_field(&sprite, LAYER, "visible", "Z").await?;
        jvm.put_field(&mut this, LAYER, "visible", "Z", visible).await?;

        Ok(())
    }

    async fn set_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        frame_width: i32,
        frame_height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::setImage({this:?}, {image:?}, {frame_width}, {frame_height})");

        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        let image_width: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
            .await?;
        let image_height: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
            .await?;
        if frame_width < 1 || frame_height < 1 || image_width % frame_width != 0 || image_height % frame_height != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid frame size").await);
        }

        let old_raw_frame_count: i32 = jvm.get_field(&this, CLASS, "rawFrameCount", "I").await?;
        let raw_frame_count = (image_width / frame_width) * (image_height / frame_height);
        // Sequence entries and the current frame may not exist in a smaller frame set.
        if raw_frame_count < old_raw_frame_count {
            jvm.put_field(&mut this, CLASS, "frameSequence", "[I", None).await?;
            jvm.put_field(&mut this, CLASS, "frame", "I", 0).await?;
        }
        jvm.put_field(&mut this, CLASS, "image", "Ljavax/microedition/lcdui/Image;", image)
            .await?;
        jvm.put_field(&mut this, CLASS, "rawFrameCount", "I", raw_frame_count).await?;

        let old_frame_width: i32 = jvm.get_field(&this, CLASS, "frameWidth", "I").await?;
        let old_frame_height: i32 = jvm.get_field(&this, CLASS, "frameHeight", "I").await?;
        if frame_width != old_frame_width || frame_height != old_frame_height {
            let transform: i32 = jvm.get_field(&this, CLASS, "transform", "I").await?;
            Self::set_geometry(jvm, &mut this, frame_width, frame_height, transform).await?;
            for (field, value) in [
                ("collisionX", 0),
                ("collisionY", 0),
                ("collisionWidth", frame_width),
                ("collisionHeight", frame_height),
            ] {
                jvm.put_field(&mut this, CLASS, field, "I", value).await?;
            }
        }

        Ok(())
    }

    async fn define_reference_pixel(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::defineReferencePixel({this:?}, {x}, {y})");

        jvm.put_field(&mut this, CLASS, "refX", "I", x).await?;
        jvm.put_field(&mut this, CLASS, "refY", "I", y).await?;

        Ok(())
    }

    async fn set_ref_pixel_position(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::setRefPixelPosition({this:?}, {x}, {y})");

        let (offset_x, offset_y) = Self::ref_pixel_offset(jvm, &this).await?;
        jvm.invoke_virtual(&this, LAYER, "setPosition", "(II)V", (x.wrapping_sub(offset_x), y.wrapping_sub(offset_y)))
            .await
    }

    async fn get_ref_pixel_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::getRefPixelX({this:?})");

        let x: i32 = jvm.get_field(&this, LAYER, "x", "I").await?;
        Ok(x.wrapping_add(Self::ref_pixel_offset(jvm, &this).await?.0))
    }

    async fn get_ref_pixel_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::getRefPixelY({this:?})");

        let y: i32 = jvm.get_field(&this, LAYER, "y", "I").await?;
        Ok(y.wrapping_add(Self::ref_pixel_offset(jvm, &this).await?.1))
    }

    async fn set_frame(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, sequence_index: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::setFrame({this:?}, {sequence_index})");

        let length: i32 = jvm.invoke_virtual(&this, CLASS, "getFrameSequenceLength", "()I", ()).await?;
        if sequence_index < 0 || sequence_index >= length {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid frame sequence index").await);
        }

        jvm.put_field(&mut this, CLASS, "frame", "I", sequence_index).await
    }

    async fn get_frame(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::getFrame({this:?})");

        jvm.get_field(&this, CLASS, "frame", "I").await
    }

    async fn get_raw_frame_count(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::getRawFrameCount({this:?})");

        jvm.get_field(&this, CLASS, "rawFrameCount", "I").await
    }

    async fn get_frame_sequence_length(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::getFrameSequenceLength({this:?})");

        let frame_sequence: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, CLASS, "frameSequence", "[I").await?;
        if frame_sequence.is_null() {
            jvm.get_field(&this, CLASS, "rawFrameCount", "I").await
        } else {
            Ok(jvm.array_length(&frame_sequence).await? as i32)
        }
    }

    async fn next_frame(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::nextFrame({this:?})");

        let length: i32 = jvm.invoke_virtual(&this, CLASS, "getFrameSequenceLength", "()I", ()).await?;
        let frame: i32 = jvm.get_field(&this, CLASS, "frame", "I").await?;
        jvm.put_field(&mut this, CLASS, "frame", "I", (frame + 1) % length).await
    }

    async fn prev_frame(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::prevFrame({this:?})");

        let length: i32 = jvm.invoke_virtual(&this, CLASS, "getFrameSequenceLength", "()I", ()).await?;
        let frame: i32 = jvm.get_field(&this, CLASS, "frame", "I").await?;
        jvm.put_field(&mut this, CLASS, "frame", "I", (frame + length - 1) % length).await
    }

    async fn set_frame_sequence(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        sequence: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::setFrameSequence({this:?}, {sequence:?})");

        jvm.put_field(&mut this, CLASS, "frame", "I", 0).await?;
        if sequence.is_null() {
            return jvm.put_field(&mut this, CLASS, "frameSequence", "[I", None).await;
        }

        let frames: Vec<i32> = jvm.load_array(&sequence, 0, jvm.array_length(&sequence).await?).await?;
        if frames.is_empty() {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Empty frame sequence").await);
        }
        let raw_frame_count: i32 = jvm.get_field(&this, CLASS, "rawFrameCount", "I").await?;
        if frames.iter().any(|frame| !(0..raw_frame_count).contains(frame)) {
            return Err(jvm.exception("java/lang/ArrayIndexOutOfBoundsException", "Invalid raw frame index").await);
        }

        // The sprite keeps its own copy so later changes to the caller's array have no effect.
        let mut copy = jvm.instantiate_array("I", frames.len()).await?;
        jvm.store_array(&mut copy, 0, frames).await?;
        jvm.put_field(&mut this, CLASS, "frameSequence", "[I", copy).await
    }

    async fn define_collision_rectangle(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::defineCollisionRectangle({this:?}, {x}, {y}, {width}, {height})");

        if width < 0 || height < 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Negative collision rectangle size")
                .await);
        }

        for (field, value) in [
            ("collisionX", x),
            ("collisionY", y),
            ("collisionWidth", width),
            ("collisionHeight", height),
        ] {
            jvm.put_field(&mut this, CLASS, field, "I", value).await?;
        }

        Ok(())
    }

    async fn set_transform(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, transform: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::setTransform({this:?}, {transform})");

        if !(0..8).contains(&transform) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid transform").await);
        }

        let frame_width: i32 = jvm.get_field(&this, CLASS, "frameWidth", "I").await?;
        let frame_height: i32 = jvm.get_field(&this, CLASS, "frameHeight", "I").await?;
        Self::set_geometry(jvm, &mut this, frame_width, frame_height, transform).await
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::paint({this:?}, {g:?})");

        if g.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "g is null").await);
        }
        let visible: bool = jvm.get_field(&this, LAYER, "visible", "Z").await?;
        if !visible {
            return Ok(());
        }

        let placement = Self::placement(jvm, &this).await?;
        jvm.invoke_virtual(
            &g,
            "javax/microedition/lcdui/Graphics",
            "drawRegion",
            "(Ljavax/microedition/lcdui/Image;IIIIIIII)V",
            [
                placement.image.into(),
                placement.frame_x.into(),
                placement.frame_y.into(),
                placement.frame_width.into(),
                placement.frame_height.into(),
                placement.transform.into(),
                placement.x.into(),
                placement.y.into(),
                (4 | 16).into(), // LEFT | TOP
            ],
        )
        .await
    }

    async fn collides_with_sprite(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        sprite: ClassInstanceRef<Sprite>,
        pixel_level: bool,
    ) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::collidesWith({this:?}, {sprite:?}, {pixel_level})");

        if sprite.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "sprite is null").await);
        }
        let visible: bool = jvm.get_field(&this, LAYER, "visible", "Z").await?;
        let other_visible: bool = jvm.get_field(&sprite, LAYER, "visible", "Z").await?;
        if !visible || !other_visible {
            return Ok(false);
        }

        let placement = Self::placement(jvm, &this).await?;
        let other = Self::placement(jvm, &sprite).await?;
        let area = placement.collision_bounds().intersect(&other.collision_bounds());
        if !pixel_level {
            return Ok(!area.is_empty());
        }

        // Parts of a collision rectangle outside the frame have no pixels to collide with.
        let area = area.intersect(&placement.bounds()).intersect(&other.bounds());
        let pixels = Image::image(jvm, &placement.image).await?;
        let other_pixels = Image::image(jvm, &other.image).await?;
        Ok(area
            .points()
            .any(|(x, y)| placement.is_opaque(&*pixels, x, y) && other.is_opaque(&*other_pixels, x, y)))
    }

    async fn collides_with_tiled_layer(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        tiled_layer: ClassInstanceRef<TiledLayer>,
        pixel_level: bool,
    ) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::collidesWith({this:?}, {tiled_layer:?}, {pixel_level})");

        if tiled_layer.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "tiled layer is null").await);
        }
        let visible: bool = jvm.get_field(&this, LAYER, "visible", "Z").await?;
        let other_visible: bool = jvm.get_field(&tiled_layer, LAYER, "visible", "Z").await?;
        if !visible || !other_visible {
            return Ok(false);
        }

        let placement = Self::placement(jvm, &this).await?;
        let tiles = TiledLayer::tiles(jvm, &tiled_layer).await?;
        let grid = Rect::new(
            tiles.x,
            tiles.y,
            tiles.columns.saturating_mul(tiles.tile_width),
            tiles.rows.saturating_mul(tiles.tile_height),
        );
        let area = placement.collision_bounds().intersect(&grid);
        if area.is_empty() {
            return Ok(false);
        }

        if !pixel_level {
            let columns = (area.left - tiles.x) / tiles.tile_width..=(area.right - 1 - tiles.x) / tiles.tile_width;
            let mut rows = (area.top - tiles.y) / tiles.tile_height..=(area.bottom - 1 - tiles.y) / tiles.tile_height;
            return Ok(rows.any(|row| columns.clone().any(|column| tiles.tile(column, row) != 0)));
        }

        let pixels = Image::image(jvm, &placement.image).await?;
        let tile_pixels = Image::image(jvm, &tiles.image).await?;
        Ok(area.intersect(&placement.bounds()).points().any(|(x, y)| {
            let (offset_x, offset_y) = (x - tiles.x, y - tiles.y);
            let tile = tiles.tile(offset_x / tiles.tile_width, offset_y / tiles.tile_height);
            if tile == 0 || !placement.is_opaque(&*pixels, x, y) {
                return false;
            }
            let (source_x, source_y) = tiles.source(tile);
            tile_pixels
                .get_pixel(source_x + offset_x % tiles.tile_width, source_y + offset_y % tiles.tile_height)
                .a
                == 0xff
        }))
    }

    async fn collides_with_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        pixel_level: bool,
    ) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.game.Sprite::collidesWith({this:?}, {image:?}, {x}, {y}, {pixel_level})");

        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        let visible: bool = jvm.get_field(&this, LAYER, "visible", "Z").await?;
        if !visible {
            return Ok(false);
        }

        let placement = Self::placement(jvm, &this).await?;
        let image_pixels = Image::image(jvm, &image).await?;
        let area = placement
            .collision_bounds()
            .intersect(&Rect::new(x, y, image_pixels.width() as i32, image_pixels.height() as i32));
        if !pixel_level {
            return Ok(!area.is_empty());
        }

        let pixels = Image::image(jvm, &placement.image).await?;
        Ok(area
            .intersect(&placement.bounds())
            .points()
            .any(|(point_x, point_y)| placement.is_opaque(&*pixels, point_x, point_y) && image_pixels.get_pixel(point_x - x, point_y - y).a == 0xff))
    }

    // Changes the frame size or transform while keeping the reference pixel where it is on screen.
    async fn set_geometry(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, frame_width: i32, frame_height: i32, transform: i32) -> JvmResult<()> {
        let (old_offset_x, old_offset_y) = Self::ref_pixel_offset(jvm, this).await?;

        jvm.put_field(this, CLASS, "frameWidth", "I", frame_width).await?;
        jvm.put_field(this, CLASS, "frameHeight", "I", frame_height).await?;
        jvm.put_field(this, CLASS, "transform", "I", transform).await?;
        let (width, height) = transformed_size(transform, frame_width, frame_height);
        jvm.put_field(this, LAYER, "width", "I", width).await?;
        jvm.put_field(this, LAYER, "height", "I", height).await?;

        let (offset_x, offset_y) = Self::ref_pixel_offset(jvm, this).await?;
        let x: i32 = jvm.get_field(this, LAYER, "x", "I").await?;
        let y: i32 = jvm.get_field(this, LAYER, "y", "I").await?;
        jvm.invoke_virtual(
            this,
            LAYER,
            "setPosition",
            "(II)V",
            (
                x.wrapping_add(old_offset_x).wrapping_sub(offset_x),
                y.wrapping_add(old_offset_y).wrapping_sub(offset_y),
            ),
        )
        .await
    }

    // Position of the reference pixel relative to the sprite's upper-left corner, after the transform.
    async fn ref_pixel_offset(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<(i32, i32)> {
        let transform: i32 = jvm.get_field(this, CLASS, "transform", "I").await?;
        let frame_width: i32 = jvm.get_field(this, CLASS, "frameWidth", "I").await?;
        let frame_height: i32 = jvm.get_field(this, CLASS, "frameHeight", "I").await?;
        let ref_x: i32 = jvm.get_field(this, CLASS, "refX", "I").await?;
        let ref_y: i32 = jvm.get_field(this, CLASS, "refY", "I").await?;

        Ok(transform_point(transform, frame_width, frame_height, ref_x, ref_y))
    }

    async fn placement(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Placement> {
        let image: ClassInstanceRef<Image> = jvm.get_field(this, CLASS, "image", "Ljavax/microedition/lcdui/Image;").await?;
        let image_width: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
            .await?;
        let frame_width: i32 = jvm.get_field(this, CLASS, "frameWidth", "I").await?;
        let frame_height: i32 = jvm.get_field(this, CLASS, "frameHeight", "I").await?;

        let frame: i32 = jvm.get_field(this, CLASS, "frame", "I").await?;
        let frame_sequence: ClassInstanceRef<Array<i32>> = jvm.get_field(this, CLASS, "frameSequence", "[I").await?;
        let raw_frame = if frame_sequence.is_null() {
            frame
        } else {
            jvm.load_array(&frame_sequence, frame as usize, 1).await?[0]
        };
        let frames_per_row = image_width / frame_width;

        Ok(Placement {
            x: jvm.get_field(this, LAYER, "x", "I").await?,
            y: jvm.get_field(this, LAYER, "y", "I").await?,
            image,
            frame_x: raw_frame % frames_per_row * frame_width,
            frame_y: raw_frame / frames_per_row * frame_height,
            frame_width,
            frame_height,
            transform: jvm.get_field(this, CLASS, "transform", "I").await?,
            collision: [
                jvm.get_field(this, CLASS, "collisionX", "I").await?,
                jvm.get_field(this, CLASS, "collisionY", "I").await?,
                jvm.get_field(this, CLASS, "collisionWidth", "I").await?,
                jvm.get_field(this, CLASS, "collisionHeight", "I").await?,
            ],
        })
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaValue, Jvm, Result as JvmResult};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use super::{CLASS, LAYER, Sprite};
    use crate::{
        classes::javax::microedition::lcdui::{
            Graphics, Image,
            game::{Layer, LayerManager, TiledLayer},
        },
        get_protos,
    };

    const TILED_LAYER: &str = "javax/microedition/lcdui/game/TiledLayer";
    const LAYER_MANAGER: &str = "javax/microedition/lcdui/game/LayerManager";
    const RED: (u8, u8, u8) = (0xff, 0, 0);
    const BLUE: (u8, u8, u8) = (0, 0, 0xff);
    const WHITE: (u8, u8, u8) = (0xff, 0xff, 0xff);

    async fn new_canvas(jvm: &Jvm) -> JvmResult<(ClassInstanceRef<Image>, ClassInstanceRef<Graphics>)> {
        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (100, 100),
            )
            .await?;
        let graphics = jvm
            .invoke_virtual(
                &image,
                "javax/microedition/lcdui/Image",
                "getGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;

        Ok((image, graphics))
    }

    async fn pixel(jvm: &Jvm, image: &ClassInstanceRef<Image>, x: i32, y: i32) -> JvmResult<(u8, u8, u8)> {
        let color = Image::image(jvm, image).await?.get_pixel(x, y);

        Ok((color.r, color.g, color.b))
    }

    #[test]
    fn sprites_and_tiled_layers_paint_and_collide() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            // Two 2x2 frames: only the upper-left pixel of the first is opaque (red), the second is solid blue.
            let (red, blue) = (0xffff0000u32 as i32, 0xff0000ffu32 as i32);
            let mut rgb = jvm.instantiate_array("I", 8).await?;
            jvm.store_array(&mut rgb, 0, [red, 0, blue, blue, 0, 0, blue, blue]).await?;
            let frames: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createRGBImage",
                    "([IIIZ)Ljavax/microedition/lcdui/Image;",
                    (rgb, 4, 2, true),
                )
                .await?;

            let sprite: ClassInstanceRef<Sprite> = jvm
                .new_class(CLASS, "(Ljavax/microedition/lcdui/Image;II)V", (frames.clone(), 2, 2))
                .await?
                .into();
            let other: ClassInstanceRef<Sprite> = jvm
                .new_class(CLASS, "(Ljavax/microedition/lcdui/game/Sprite;)V", (sprite.clone(),))
                .await?
                .into();
            assert_eq!(jvm.invoke_virtual::<_, i32>(&sprite, CLASS, "getRawFrameCount", "()I", ()).await?, 2);

            // Mirroring keeps the reference pixel at (10, 10), which moves the sprite one pixel to the left.
            let _: () = jvm.invoke_virtual(&sprite, LAYER, "setPosition", "(II)V", (10, 10)).await?;
            let _: () = jvm.invoke_virtual(&sprite, CLASS, "setTransform", "(I)V", (2,)).await?;
            assert_eq!(jvm.invoke_virtual::<_, i32>(&sprite, LAYER, "getX", "()I", ()).await?, 9);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&sprite, CLASS, "getRefPixelX", "()I", ()).await?, 10);

            // The mirrored sprite is opaque only at (10, 10); the other sprite only at its upper-left corner.
            for (x, pixel_level, expected) in [(10, true, true), (11, false, false), (9, false, true), (9, true, false)] {
                let _: () = jvm.invoke_virtual(&other, LAYER, "setPosition", "(II)V", (x, 10)).await?;
                let collides: bool = jvm
                    .invoke_virtual(
                        &sprite,
                        CLASS,
                        "collidesWith",
                        "(Ljavax/microedition/lcdui/game/Sprite;Z)Z",
                        (other.clone(), pixel_level),
                    )
                    .await?;
                assert_eq!(collides, expected, "x={x} pixel_level={pixel_level}");
            }
            let _: () = jvm.invoke_virtual(&sprite, CLASS, "nextFrame", "()V", ()).await?;
            assert!(
                jvm.invoke_virtual::<_, bool>(
                    &sprite,
                    CLASS,
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/game/Sprite;Z)Z",
                    (other.clone(), true)
                )
                .await?
            );

            // Sequence [1, 0]: stepping back from entry 0 wraps to entry 1, which is the first raw frame.
            let mut sequence = jvm.instantiate_array("I", 2).await?;
            jvm.store_array(&mut sequence, 0, [1, 0]).await?;
            let _: () = jvm.invoke_virtual(&sprite, CLASS, "setFrameSequence", "([I)V", (sequence,)).await?;
            let _: () = jvm.invoke_virtual(&sprite, CLASS, "prevFrame", "()V", ()).await?;
            assert_eq!(jvm.invoke_virtual::<_, i32>(&sprite, CLASS, "getFrame", "()I", ()).await?, 1);

            let (canvas, graphics) = new_canvas(&jvm).await?;
            let _: () = jvm
                .invoke_virtual(&sprite, CLASS, "paint", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics.clone(),))
                .await?;
            assert_eq!(pixel(&jvm, &canvas, 10, 10).await?, RED);
            assert_eq!(pixel(&jvm, &canvas, 9, 10).await?, WHITE);

            // Three cells at (20, 10): empty, solid blue, and an animated tile showing the red corner tile.
            let layer: ClassInstanceRef<TiledLayer> = jvm
                .new_class(TILED_LAYER, "(IILjavax/microedition/lcdui/Image;II)V", (3, 1, frames.clone(), 2, 2))
                .await?
                .into();
            let _: () = jvm.invoke_virtual(&layer, LAYER, "setPosition", "(II)V", (20, 10)).await?;
            let _: () = jvm.invoke_virtual(&layer, TILED_LAYER, "setCell", "(III)V", (1, 0, 2)).await?;
            let animated: i32 = jvm.invoke_virtual(&layer, TILED_LAYER, "createAnimatedTile", "(I)I", (1,)).await?;
            assert_eq!(animated, -1);
            let _: () = jvm.invoke_virtual(&layer, TILED_LAYER, "setCell", "(III)V", (2, 0, animated)).await?;
            assert_eq!(jvm.invoke_virtual::<_, i32>(&layer, TILED_LAYER, "getCell", "(II)I", (2, 0)).await?, -1);

            let _: () = jvm
                .invoke_virtual(
                    &layer,
                    TILED_LAYER,
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    (graphics.clone(),),
                )
                .await?;
            for (x, expected) in [(20, WHITE), (22, BLUE), (24, RED), (25, WHITE)] {
                assert_eq!(pixel(&jvm, &canvas, x, 10).await?, expected, "x={x}");
            }
            let _: () = jvm.invoke_virtual(&layer, TILED_LAYER, "setAnimatedTile", "(II)V", (animated, 2)).await?;
            let _: () = jvm
                .invoke_virtual(&layer, TILED_LAYER, "paint", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics,))
                .await?;
            assert_eq!(pixel(&jvm, &canvas, 25, 10).await?, BLUE);

            // At x = 21 the other sprite's only opaque pixel is over the empty cell; at x = 22 it is over the blue one.
            for (x, pixel_level, expected) in [(17, false, false), (21, false, true), (21, true, false), (22, true, true)] {
                let _: () = jvm.invoke_virtual(&other, LAYER, "setPosition", "(II)V", (x, 10)).await?;
                let collides: bool = jvm
                    .invoke_virtual(
                        &other,
                        CLASS,
                        "collidesWith",
                        "(Ljavax/microedition/lcdui/game/TiledLayer;Z)Z",
                        (layer.clone(), pixel_level),
                    )
                    .await?;
                assert_eq!(collides, expected, "x={x} pixel_level={pixel_level}");
            }
            // The image's red corner is at its upper-left, so it meets the sprite's opaque pixel only at x = 22.
            for (x, pixel_level, expected) in [(22, true, true), (23, false, true), (23, true, false)] {
                let collides: bool = jvm
                    .invoke_virtual(
                        &other,
                        CLASS,
                        "collidesWith",
                        "(Ljavax/microedition/lcdui/Image;IIZ)Z",
                        (frames.clone(), x, 10, pixel_level),
                    )
                    .await?;
                assert_eq!(collides, expected, "x={x} pixel_level={pixel_level}");
            }

            // The sprite at (22, 10) is above the tiled layer; the 4x2 view window at (20, 10) is drawn at (50, 50).
            let manager: ClassInstanceRef<LayerManager> = jvm.new_class(LAYER_MANAGER, "()V", ()).await?.into();
            for appended in [JavaValue::from(other.clone()), JavaValue::from(layer.clone())] {
                let _: () = jvm
                    .invoke_virtual(&manager, LAYER_MANAGER, "append", "(Ljavax/microedition/lcdui/game/Layer;)V", [appended])
                    .await?;
            }
            assert_eq!(jvm.invoke_virtual::<_, i32>(&manager, LAYER_MANAGER, "getSize", "()I", ()).await?, 2);
            let top: ClassInstanceRef<Layer> = jvm
                .invoke_virtual(&manager, LAYER_MANAGER, "getLayerAt", "(I)Ljavax/microedition/lcdui/game/Layer;", (0,))
                .await?;
            assert_eq!(top.identity(), other.identity());

            let _: () = jvm
                .invoke_virtual(&manager, LAYER_MANAGER, "setViewWindow", "(IIII)V", (20, 10, 4, 2))
                .await?;
            let (canvas, graphics) = new_canvas(&jvm).await?;
            let _: () = jvm
                .invoke_virtual(
                    &manager,
                    LAYER_MANAGER,
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;II)V",
                    (graphics.clone(), 50, 50),
                )
                .await?;
            for (x, expected) in [(50, WHITE), (52, RED), (53, BLUE), (54, WHITE)] {
                assert_eq!(pixel(&jvm, &canvas, x, 50).await?, expected, "x={x}");
            }
            for (getter, expected) in [("getTranslateX", 0), ("getClipX", 0), ("getClipWidth", 100)] {
                let value: i32 = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", getter, "()I", ())
                    .await?;
                assert_eq!(value, expected, "{getter}");
            }

            Ok(())
        })
    }
}
