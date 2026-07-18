use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.skt.m.BackLight
pub struct BackLight;

impl BackLight {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/BackLight",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("getColor", "()I", Self::get_color, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "getColorNum",
                    "()I",
                    Self::get_color_num,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getColors",
                    "()[I",
                    Self::get_colors,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("off", "()V", Self::off, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new("on", "(I)V", Self::on, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new("setColor", "(I)V", Self::set_color, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn get_color(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.BackLight::getColor()");

        Ok(0)
    }

    async fn get_color_num(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.BackLight::getColorNum()");

        Ok(0)
    }

    async fn get_colors(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Array<i32>>> {
        tracing::warn!("stub com.skt.m.BackLight::getColors()");

        Ok(jvm.instantiate_array("I", 0).await?.into())
    }

    async fn off(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.BackLight::off()");

        Ok(())
    }

    async fn on(_jvm: &Jvm, _: &mut WieJvmContext, timeout: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.BackLight::on({timeout:?})");

        Ok(())
    }

    async fn set_color(_jvm: &Jvm, _context: &mut WieJvmContext, index: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.BackLight::setColor({index})");

        Ok(())
    }
}
