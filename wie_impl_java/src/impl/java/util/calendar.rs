use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
    JavaContext, JavaMethodFlag, JavaResult,
};

// class java.util.Calendar
pub struct Calendar {}

impl Calendar {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "getInstance",
                "()Ljava/util/Calendar;",
                Self::get_instance,
                JavaMethodFlag::STATIC,
            )],
            fields: vec![],
        }
    }

    async fn get_instance(context: &mut dyn JavaContext) -> JavaResult<JvmClassInstanceProxy<Calendar>> {
        tracing::warn!("stub java.util.Calendar::getInstance()");

        let instance = context.jvm().instantiate_class("java/util/GregorianCalendar").await?;
        // TODO call <init>

        Ok(JvmClassInstanceProxy::new(instance))
    }
}
