use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.handset.BackLight
#[java_class(name = "org/kwis/msp/handset/BackLight", public)]
pub struct BackLight;

#[java_methods]
impl BackLight {
    #[method(public, static)]
    async fn always_on(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.BackLight::alwaysOn()");

        Ok(())
    }

    #[method(public, static)]
    async fn on(_: &Jvm, _: &mut WieJvmContext, id: i32, color: i32, duration: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.BackLight::on({id}, {color}, {duration})");

        Ok(())
    }

    #[method(public, static)]
    async fn off(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.BackLight::off()");

        Ok(())
    }

    #[method(public, static)]
    async fn before(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.BackLight::before()");

        Ok(())
    }
}
