use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

// class com.skt.m.SISImage
pub struct SISImage;

impl SISImage {
    pub fn as_proto() -> WieJavaClassProto {
        let public = MethodAccessFlags::PUBLIC;
        let public_static = MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC;
        let public_static_final = FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL;

        WieJavaClassProto {
            name: "com/skt/m/SISImage",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new("createBuffer", "(II)V", Self::create_buffer, public_static),
                JavaMethodProto::new(
                    "createSISImage",
                    "([BII)Lcom/skt/m/SISImage;",
                    Self::create_sis_image_from_data,
                    public_static,
                ),
                JavaMethodProto::new(
                    "createSISImage",
                    "(Ljava/lang/String;)Lcom/skt/m/SISImage;",
                    Self::create_sis_image_from_name,
                    public_static,
                ),
                JavaMethodProto::new("freeBuffer", "()V", Self::free_buffer, public_static),
                JavaMethodProto::new("getRequiredBufferSize", "([BII)I", Self::get_required_buffer_size, public_static),
                JavaMethodProto::new("getBestID", "()I", Self::get_best_id, public),
                JavaMethodProto::new("getDelay", "(I)I", Self::get_delay, public),
                JavaMethodProto::new("getFrame", "(I)Ljavax/microedition/lcdui/Image;", Self::get_frame, public),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, public),
                JavaMethodProto::new("getImageLevel", "()I", Self::get_image_level, public),
                JavaMethodProto::new("getMaxFrameID", "()I", Self::get_max_frame_id, public),
                JavaMethodProto::new("getMaxObjectID", "()I", Self::get_max_object_id, public),
                JavaMethodProto::new("getObject", "(IZ)Ljavax/microedition/lcdui/Image;", Self::get_object, public),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, public),
                JavaMethodProto::new("paintFrame", "(Ljavax/microedition/lcdui/Graphics;III)V", Self::paint_frame, public),
                JavaMethodProto::new("paintObject", "(Ljavax/microedition/lcdui/Graphics;IIIZ)V", Self::paint_object, public),
            ],
            fields: ["IMG_LEVEL_BW", "IMG_LEVEL_4G", "IMG_LEVEL_256C"]
                .into_iter()
                .map(|name| JavaFieldProto::new(name, "I", public_static_final))
                .collect(),
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.skt.m.SISImage::<clinit>()");

        for (name, value) in [("IMG_LEVEL_BW", 1), ("IMG_LEVEL_4G", 2), ("IMG_LEVEL_256C", 8)] {
            jvm.put_static_field("com/skt/m/SISImage", name, "I", value).await?;
        }

        Ok(())
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.SISImage::<init>({this:?})");
        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn create_buffer(jvm: &Jvm, _context: &mut WieJvmContext, object_buffer_size: i32, other_buffer_size: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.SISImage::createBuffer({object_buffer_size}, {other_buffer_size})");

        if object_buffer_size <= 0 || other_buffer_size <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "buffer sizes must be positive").await);
        }

        Ok(())
    }

    async fn create_sis_image_from_data(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub com.skt.m.SISImage::createSISImage({data:?}, {offset}, {length})");

        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "data is null").await);
        }

        let array_length = jvm.array_length(&data).await?;
        if offset < 0 || length < 0 || (offset as usize).checked_add(length as usize).is_none_or(|end| end > array_length) {
            return Err(jvm
                .exception("java/lang/ArrayIndexOutOfBoundsException", "invalid offset or length")
                .await);
        }

        Ok(jvm.new_class("com/skt/m/SISImage", "()V", ()).await?.into())
    }

    async fn create_sis_image_from_name(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub com.skt.m.SISImage::createSISImage({name:?})");

        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "name is null").await);
        }

        Ok(jvm.new_class("com/skt/m/SISImage", "()V", ()).await?.into())
    }

    async fn free_buffer(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.SISImage::freeBuffer()");
        Ok(())
    }

    async fn get_required_buffer_size(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.SISImage::getRequiredBufferSize({data:?}, {offset}, {length})");

        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "data is null").await);
        }

        let array_length = jvm.array_length(&data).await?;
        if offset < 0 || length < 0 || (offset as usize).checked_add(length as usize).is_none_or(|end| end > array_length) {
            return Err(jvm
                .exception("java/lang/ArrayIndexOutOfBoundsException", "invalid offset or length")
                .await);
        }

        Ok(0)
    }

    async fn get_best_id(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.SISImage::getBestID({this:?})");
        Ok(0)
    }

    async fn get_delay(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, frame_id: i32) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.SISImage::getDelay({this:?}, {frame_id})");
        Ok(0)
    }

    async fn get_frame(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, frame_id: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub com.skt.m.SISImage::getFrame({this:?}, {frame_id})");

        if frame_id != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid frame ID").await);
        }

        Ok(ClassInstanceRef::new(None))
    }

    async fn get_height(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.SISImage::getHeight({this:?})");
        Ok(0)
    }

    async fn get_image_level(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.SISImage::getImageLevel({this:?})");
        Ok(0)
    }

    async fn get_max_frame_id(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.SISImage::getMaxFrameID({this:?})");
        Ok(0)
    }

    async fn get_max_object_id(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.SISImage::getMaxObjectID({this:?})");
        Ok(0)
    }

    async fn get_object(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        object_id: i32,
        use_transparency: bool,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub com.skt.m.SISImage::getObject({this:?}, {object_id}, {use_transparency})");

        if object_id != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid object ID").await);
        }

        Ok(ClassInstanceRef::new(None))
    }

    async fn get_width(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.SISImage::getWidth({this:?})");
        Ok(0)
    }

    async fn paint_frame(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        frame_id: i32,
        x: i32,
        y: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.SISImage::paintFrame({this:?}, {graphics:?}, {frame_id}, {x}, {y})");

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        if frame_id != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid frame ID").await);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn paint_object(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        object_id: i32,
        x: i32,
        y: i32,
        use_transparency: bool,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.SISImage::paintObject({this:?}, {graphics:?}, {object_id}, {x}, {y}, {use_transparency})");

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        if object_id != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid object ID").await);
        }

        Ok(())
    }
}
