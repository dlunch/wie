use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::Graphics;

use super::Object3D;

// class com.skt.m3d.Graphics3D
pub struct Graphics3D;

impl Graphics3D {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m3d/Graphics3D",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "clearZBuffer",
                    "()V",
                    Self::clear_z_buffer,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "destroyZBuffer",
                    "()V",
                    Self::destroy_z_buffer,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "drawWireframe",
                    "(Ljavax/microedition/lcdui/Graphics;Lcom/skt/m3d/Object3D;)V",
                    Self::draw_wireframe,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "isBackfaceCulled",
                    "()Z",
                    Self::is_backface_culled,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "isZBufferEnabled",
                    "()Z",
                    Self::is_z_buffer_enabled,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "render",
                    "(Ljavax/microedition/lcdui/Graphics;Lcom/skt/m3d/Object3D;)V",
                    Self::render,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setBackfaceCulled",
                    "(Z)V",
                    Self::set_backface_culled,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setZBufferEnabled",
                    "(Z)V",
                    Self::set_z_buffer_enabled,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("backfaceCulled", "Z", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                JavaFieldProto::new("zBufferEnabled", "Z", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        jvm.put_static_field("com/skt/m3d/Graphics3D", "backfaceCulled", "Z", false).await?;
        jvm.put_static_field("com/skt/m3d/Graphics3D", "zBufferEnabled", "Z", false).await
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.skt.m3d.Graphics3D::<init>({this:?})");
        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn clear_z_buffer(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::clearZBuffer()");
        Ok(())
    }

    async fn destroy_z_buffer(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::destroyZBuffer()");
        Ok(())
    }

    async fn draw_wireframe(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        object: ClassInstanceRef<Object3D>,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::drawWireframe({graphics:?}, {object:?})");

        if graphics.is_null() || object.is_null() {
            return Err(jvm
                .exception("java/lang/NullPointerException", "graphics and object must not be null")
                .await);
        }

        Ok(())
    }

    async fn is_backface_culled(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        jvm.get_static_field("com/skt/m3d/Graphics3D", "backfaceCulled", "Z").await
    }

    async fn is_z_buffer_enabled(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        jvm.get_static_field("com/skt/m3d/Graphics3D", "zBufferEnabled", "Z").await
    }

    async fn render(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        object: ClassInstanceRef<Object3D>,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::render({graphics:?}, {object:?})");

        if graphics.is_null() || object.is_null() {
            return Err(jvm
                .exception("java/lang/NullPointerException", "graphics and object must not be null")
                .await);
        }

        Ok(())
    }

    async fn set_backface_culled(jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::setBackfaceCulled({enabled})");
        jvm.put_static_field("com/skt/m3d/Graphics3D", "backfaceCulled", "Z", enabled).await
    }

    async fn set_z_buffer_enabled(jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::setZBufferEnabled({enabled})");
        jvm.put_static_field("com/skt/m3d/Graphics3D", "zBufferEnabled", "Z", enabled).await
    }
}
