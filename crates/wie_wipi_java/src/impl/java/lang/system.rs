use alloc::vec;

use crate::base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class java.lang.System
pub struct System {}

impl System {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("currentTimeMillis", "()J", Self::current_time_millis)],
            fields: vec![],
        }
    }

    fn current_time_millis(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::debug!("System::currentTimeMillis()");

        Ok(0)
    }
}
