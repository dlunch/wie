use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.handset.LED
#[java_class(name = "org/kwis/msp/handset/LED", public)]
pub struct LED;

#[java_methods]
impl LED {
    #[method(public, static)]
    async fn set(_: &Jvm, _: &mut WieJvmContext, leds: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.LED::set({leds})");

        Ok(())
    }
}
