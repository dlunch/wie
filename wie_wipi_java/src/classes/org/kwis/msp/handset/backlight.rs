use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.handset.BackLight
pub struct BackLight;

impl BackLight {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/handset/BackLight",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("alwaysOn", "()V", Self::always_on, MethodAccessFlags::STATIC),
                JavaMethodProto::new("on", "(III)V", Self::on, MethodAccessFlags::STATIC),
                JavaMethodProto::new("off", "()V", Self::off, MethodAccessFlags::STATIC),
                JavaMethodProto::new("before", "()V", Self::before, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn always_on(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.BackLight::alwaysOn()");

        Ok(())
    }

    async fn on(_: &Jvm, _: &mut WieJvmContext, id: i32, color: i32, duration: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.BackLight::on({id}, {color}, {duration})");

        Ok(())
    }

    async fn off(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.BackLight::off()");

        Ok(())
    }

    async fn before(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.BackLight::before()");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    #[test]
    fn test_backlight_stubs_are_callable() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let _: () = jvm
                .invoke_static("org/kwis/msp/handset/BackLight", "on", "(III)V", (1, 0xffffff, 1000))
                .await?;
            let _: () = jvm.invoke_static("org/kwis/msp/handset/BackLight", "off", "()V", ()).await?;
            let _: () = jvm.invoke_static("org/kwis/msp/handset/BackLight", "before", "()V", ()).await?;

            Ok(())
        })
    }
}
