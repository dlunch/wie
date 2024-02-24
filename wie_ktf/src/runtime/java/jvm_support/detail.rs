use alloc::{boxed::Box, format};

use jvm::{ClassDefinition, Jvm, JvmDetail, Result as JvmResult};

use wie_core_arm::ArmCore;

use super::array_class_definition::JavaArrayClassDefinition;

pub struct KtfJvmDetail {
    core: ArmCore,
}

impl KtfJvmDetail {
    pub fn new(core: &ArmCore) -> Self {
        Self { core: core.clone() }
    }
}

#[async_trait::async_trait(?Send)]
impl JvmDetail for KtfJvmDetail {
    async fn define_class(&self, _jvm: &Jvm, _name: &str, _data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        unimplemented!()
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        let class_name = format!("[{}", element_type_name);
        let class = JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, &class_name).await.unwrap();

        Ok(Box::new(class) as Box<_>)
    }
}
