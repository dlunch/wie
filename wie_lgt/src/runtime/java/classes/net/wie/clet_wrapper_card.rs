use alloc::{format, vec};

use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
use jvm::{ClassInstanceRef, JavaError, Jvm, Result as JvmResult};

use wie_midp::classes::javax::microedition::lcdui::Graphics;
use wie_util::WieError;

use super::CletWrapperContext;

// class net.wie.CletWrapperCard
pub struct CletWrapperCard;

impl CletWrapperCard {
    pub fn as_proto() -> JavaClassProto<CletWrapperContext> {
        JavaClassProto {
            name: "net/wie/CletWrapperCard",
            parent_class: Some("org/kwis/msp/lcdui/Card"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(II)V", Self::init, Default::default()),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, Default::default()),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, Default::default()),
                JavaMethodProto::new("notifyEvent", "(III)V", Self::notify_event, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("paintClet", "I", Default::default()),
                JavaFieldProto::new("handleCletEvent", "I", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut CletWrapperContext,
        mut this: ClassInstanceRef<Self>,
        paint_clet: i32,
        handle_clet_event: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "net.wie.CletWrapperCard::<init>({:?}, {:#x}, {:#x})",
            &this,
            paint_clet,
            handle_clet_event
        );

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "paintClet", "I", paint_clet).await?;
        jvm.put_field(&mut this, "handleCletEvent", "I", handle_clet_event).await?;

        Ok(())
    }

    async fn paint(
        jvm: &Jvm,
        context: &mut CletWrapperContext,
        this: ClassInstanceRef<Self>,
        _graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapperCard::paint({:?})", &this);

        let paint_clet: i32 = jvm.get_field(&this, "paintClet", "I").await?;
        let _: () = context.core.run_function(paint_clet as _, &[]).await.map_err(|x| match x {
            WieError::FatalError(x) => JavaError::FatalError(x),
            _ => JavaError::FatalError(format!("{x}")),
        })?;

        Ok(())
    }

    async fn key_notify(jvm: &Jvm, context: &mut CletWrapperContext, this: ClassInstanceRef<Self>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("net.wie.CletWrapperCard::keyNotify({:?}, {}, {})", &this, r#type, key);

        let handle_clet_event: i32 = jvm.get_field(&this, "handleCletEvent", "I").await?;
        let r#type = r#type + 501; // TODO constants
        let _: () = context
            .core
            .run_function(handle_clet_event as _, &[r#type as _, key as _, 0 as _])
            .await
            .map_err(|x| match x {
                WieError::FatalError(x) => JavaError::FatalError(x),
                _ => JavaError::FatalError(format!("{x}")),
            })?;

        Ok(true)
    }

    async fn notify_event(
        jvm: &Jvm,
        context: &mut CletWrapperContext,
        this: ClassInstanceRef<Self>,
        r#type: i32,
        param1: i32,
        param2: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapperCard::notifyEvent({this:?}, {}, {param1}, {param2})", r#type);

        let handle_clet_event: i32 = jvm.get_field(&this, "handleCletEvent", "I").await?;
        let _: () = context
            .core
            .run_function(handle_clet_event as _, &[r#type as _, param1 as _, param2 as _])
            .await
            .map_err(|x| match x {
                WieError::FatalError(x) => JavaError::FatalError(x),
                _ => JavaError::FatalError(format!("{x}")),
            })?;

        Ok(())
    }
}
