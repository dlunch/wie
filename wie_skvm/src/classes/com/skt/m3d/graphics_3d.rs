use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::Graphics;

use super::Object3D;

// class com.skt.m3d.Graphics3D
pub struct Graphics3D;

impl Graphics3D {
    pub fn as_proto() -> WieJavaClassProto {
        let public_static = MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC;

        WieJavaClassProto {
            name: "com/skt/m3d/Graphics3D",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("clearZBuffer", "()V", Self::clear_z_buffer, public_static),
                JavaMethodProto::new("destroyZBuffer", "()V", Self::destroy_z_buffer, public_static),
                JavaMethodProto::new(
                    "drawWireframe",
                    "(Ljavax/microedition/lcdui/Graphics;Lcom/skt/m3d/Object3D;)V",
                    Self::draw_wireframe,
                    public_static,
                ),
                JavaMethodProto::new("isBackfaceCulled", "()Z", Self::is_backface_culled, public_static),
                JavaMethodProto::new("isZBufferEnabled", "()Z", Self::is_z_buffer_enabled, public_static),
                JavaMethodProto::new(
                    "render",
                    "(Ljavax/microedition/lcdui/Graphics;Lcom/skt/m3d/Object3D;)V",
                    Self::render,
                    public_static,
                ),
                JavaMethodProto::new("setBackfaceCulled", "(Z)V", Self::set_backface_culled, public_static),
                JavaMethodProto::new("setZBufferEnabled", "(Z)V", Self::set_z_buffer_enabled, public_static),
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

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

    use super::{Graphics3D, Object3D};

    #[test]
    fn flags_round_trip_and_render_operations_are_no_op() {
        let result = run_jvm_test(
            Box::new([wie_midp::get_protos().into(), Box::new([Graphics3D::as_proto(), Object3D::as_proto()])]),
            |jvm| async move {
                let _: ClassInstanceRef<Graphics3D> = jvm.new_class("com/skt/m3d/Graphics3D", "()V", ()).await?.into();

                let z_buffer_enabled: bool = jvm.invoke_static("com/skt/m3d/Graphics3D", "isZBufferEnabled", "()Z", ()).await?;
                let backface_culled: bool = jvm.invoke_static("com/skt/m3d/Graphics3D", "isBackfaceCulled", "()Z", ()).await?;
                assert!(!z_buffer_enabled);
                assert!(!backface_culled);

                let _: () = jvm.invoke_static("com/skt/m3d/Graphics3D", "setZBufferEnabled", "(Z)V", (true,)).await?;
                let _: () = jvm.invoke_static("com/skt/m3d/Graphics3D", "setBackfaceCulled", "(Z)V", (true,)).await?;
                assert!(
                    jvm.invoke_static::<_, bool>("com/skt/m3d/Graphics3D", "isZBufferEnabled", "()Z", ())
                        .await?
                );
                assert!(
                    jvm.invoke_static::<_, bool>("com/skt/m3d/Graphics3D", "isBackfaceCulled", "()Z", ())
                        .await?
                );

                let _: () = jvm.invoke_static("com/skt/m3d/Graphics3D", "clearZBuffer", "()V", ()).await?;
                let _: () = jvm.invoke_static("com/skt/m3d/Graphics3D", "destroyZBuffer", "()V", ()).await?;
                assert!(
                    jvm.invoke_static::<_, bool>("com/skt/m3d/Graphics3D", "isZBufferEnabled", "()Z", ())
                        .await?
                );

                let image: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (2, 2),
                    )
                    .await?;
                let graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(&image, "getGraphics", "()Ljavax/microedition/lcdui/Graphics;", ())
                    .await?;
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "triangle").await?.into();
                let object: ClassInstanceRef<Object3D> = jvm.new_class("com/skt/m3d/Object3D", "(Ljava/lang/String;)V", (name,)).await?.into();

                let _: () = jvm
                    .invoke_static(
                        "com/skt/m3d/Graphics3D",
                        "render",
                        "(Ljavax/microedition/lcdui/Graphics;Lcom/skt/m3d/Object3D;)V",
                        (graphics.clone(), object.clone()),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_static(
                        "com/skt/m3d/Graphics3D",
                        "drawWireframe",
                        "(Ljavax/microedition/lcdui/Graphics;Lcom/skt/m3d/Object3D;)V",
                        (graphics, object),
                    )
                    .await?;

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
