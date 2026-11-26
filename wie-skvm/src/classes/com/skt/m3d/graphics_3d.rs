use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::lcdui::Graphics;

use super::Object3D;

// class com.skt.m3d.Graphics3D
#[java_class(name = "com/skt/m3d/Graphics3D", public)]
pub struct Graphics3D {
    #[field(private, static)]
    backfaceCulled: bool,
    #[field(private, static)]
    zBufferEnabled: bool,
}

#[java_methods]
impl Graphics3D {
    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        jvm.put_static_field("com/skt/m3d/Graphics3D", "backfaceCulled", "Z", false).await?;
        jvm.put_static_field("com/skt/m3d/Graphics3D", "zBufferEnabled", "Z", false).await
    }

    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.skt.m3d.Graphics3D::<init>({this:?})");
        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    #[method(public, static)]
    async fn clear_z_buffer(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::clearZBuffer()");
        Ok(())
    }

    #[method(public, static)]
    async fn destroy_z_buffer(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::destroyZBuffer()");
        Ok(())
    }

    #[method(public, static)]
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

    #[method(public, static)]
    async fn is_backface_culled(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        jvm.get_static_field("com/skt/m3d/Graphics3D", "backfaceCulled", "Z").await
    }

    #[method(public, static)]
    async fn is_z_buffer_enabled(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        jvm.get_static_field("com/skt/m3d/Graphics3D", "zBufferEnabled", "Z").await
    }

    #[method(public, static)]
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

    #[method(public, static)]
    async fn set_backface_culled(jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::setBackfaceCulled({enabled})");
        jvm.put_static_field("com/skt/m3d/Graphics3D", "backfaceCulled", "Z", enabled).await
    }

    #[method(public, static)]
    async fn set_z_buffer_enabled(jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m3d.Graphics3D::setZBufferEnabled({enabled})");
        jvm.put_static_field("com/skt/m3d/Graphics3D", "zBufferEnabled", "Z", enabled).await
    }
}
