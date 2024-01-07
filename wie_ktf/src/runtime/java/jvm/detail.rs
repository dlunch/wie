use alloc::{boxed::Box, format};

use jvm::{ArrayClass, Class, JvmDetail, JvmResult, ThreadContext, ThreadId};

use wie_backend::SystemHandle;
use wie_core_arm::ArmCore;

use super::{array_class::JavaArrayClass, class_loader::ClassLoader, context_data::JavaContextData};

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
    async fn load_class(&mut self, class_name: &str) -> JvmResult<Option<Box<dyn Class>>> {
        let class = ClassLoader::get_or_load_class(&mut self.core, &mut self.system, class_name).await?;

        Ok(class.map(|x| Box::new(x) as _))
    }

    async fn load_array_class(&mut self, element_type_name: &str) -> JvmResult<Option<Box<dyn ArrayClass>>> {
        let class_name = format!("[{}", element_type_name);
        if let Some(x) = JavaContextData::find_class(&self.core, &class_name)? {
            let class = JavaArrayClass::from_raw(x.ptr_raw, &self.core);
            return Ok(Some(Box::new(class)));
        } else {
            let class = JavaArrayClass::new(&mut self.core, &mut self.system, &class_name).await?;

            Ok(Some(Box::new(class)))
        }
    }

    fn get_class(&self, class_name: &str) -> JvmResult<Option<Box<dyn Class>>> {
        let class = JavaContextData::find_class(&self.core, class_name)?;

        Ok(class.map(|x| Box::new(x) as _))
    }

    fn thread_context(&mut self, _thread_id: ThreadId) -> &mut dyn ThreadContext {
        todo!()
    }
}
