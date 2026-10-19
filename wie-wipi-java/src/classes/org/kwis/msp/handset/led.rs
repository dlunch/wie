use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.handset.LED
pub struct LED;

impl LED {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/handset/LED",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "set",
                "(I)V",
                Self::set,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn set(_: &Jvm, _: &mut WieJvmContext, leds: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.LED::set({leds})");

        Ok(())
    }
}
