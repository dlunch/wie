use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
    r#impl::java::lang::String,
    JavaContext, JavaMethodFlag, JavaResult,
};

// class java.lang.Integer
pub struct Integer {}

impl Integer {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "parseInt",
                "(Ljava/lang/String;)I",
                Self::parse_int,
                JavaMethodFlag::STATIC,
            )],
            fields: vec![],
        }
    }

    async fn parse_int(context: &mut dyn JavaContext, s: JvmClassInstanceProxy<String>) -> JavaResult<i32> {
        tracing::debug!("java.lang.Integer::parseInt({:?})", &s);

        let s = String::to_rust_string(context, &s)?;

        Ok(s.parse()?)
    }
}
