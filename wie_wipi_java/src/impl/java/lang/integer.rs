use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    r#impl::java::lang::String,
    JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
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

    async fn parse_int(context: &mut dyn JavaContext, s: JavaObjectProxy<String>) -> JavaResult<i32> {
        tracing::debug!("java.lang.Integer::parseInt({:#x})", s.ptr_instance);

        let s = String::to_rust_string(context, &s)?;

        Ok(s.parse()?)
    }
}
