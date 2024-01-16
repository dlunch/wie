use alloc::{boxed::Box, format};

use jvm::{Class, JvmDetail, JvmResult, ThreadContext, ThreadId};

use wie_backend::SystemHandle;
use wie_core_arm::ArmCore;

use super::array_class::JavaArrayClass;

pub struct KtfJvmDetail {
    core: ArmCore,
    system: SystemHandle,
}

impl KtfJvmDetail {
    pub fn new(core: &ArmCore, system: &SystemHandle) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl JvmDetail for KtfJvmDetail {
    async fn define_class(&self, _name: &str, _data: &[u8]) -> JvmResult<Box<dyn Class>> {
        unimplemented!()
    }

    async fn define_array_class(&self, element_type_name: &str) -> JvmResult<Box<dyn Class>> {
        let class_name = format!("[{}", element_type_name);
        let class = JavaArrayClass::new(&mut self.core.clone(), &mut self.system.clone(), &class_name).await?;

        Ok(Box::new(class) as Box<_>)
    }

    fn thread_context(&mut self, _thread_id: ThreadId) -> Box<dyn ThreadContext> {
        todo!()
    }
}
