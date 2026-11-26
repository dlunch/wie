use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class com.skt.m.BackLight
#[java_class(name = "com/skt/m/BackLight", public, final)]
pub struct BackLight;

#[java_methods]
impl BackLight {
    #[method(public, static)]
    async fn get_color(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.BackLight::getColor()");

        Ok(0)
    }

    #[method(public, static)]
    async fn get_color_num(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.BackLight::getColorNum()");

        Ok(0)
    }

    #[method(public, static)]
    async fn get_colors(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::warn!("stub com.skt.m.BackLight::getColors()");

        Ok(jvm.instantiate_array("I", 0).await?.into())
    }

    #[method(public, static)]
    async fn off(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.BackLight::off()");

        Ok(())
    }

    #[method(public, static)]
    async fn on(_jvm: &Jvm, _: &mut WieJvmContext, timeout: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.BackLight::on({timeout:?})");

        Ok(())
    }

    #[method(public, static)]
    async fn set_color(_jvm: &Jvm, _context: &mut WieJvmContext, index: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.BackLight::setColor({index})");

        Ok(())
    }
}
