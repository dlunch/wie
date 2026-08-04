use alloc::{boxed::Box, format, string::String};
use core::fmt::{self, Debug, Formatter};

use jvm::{ArrayClassDefinition, ClassDefinition, ClassInstance, JavaType, Jvm, Result as JvmResult};

use wie_core_arm::ArmCore;
use wie_jvm_support::native::array_element_size;
use wie_util::Result;

use crate::runtime::java::JavaSvcFunctions;

use super::{JavaArrayClassInstance, JavaClassDefinition};

#[derive(Clone)]
pub struct JavaArrayClassDefinition {
    pub class: JavaClassDefinition,
    core: ArmCore,
}

impl JavaArrayClassDefinition {
    pub async fn new(core: &mut ArmCore, jvm: &Jvm, element_type_name: &str, functions: JavaSvcFunctions) -> Result<Self> {
        let class = JavaClassDefinition::new_array(core, jvm, &format!("[{element_type_name}"), functions).await?;
        Ok(Self { class, core: core.clone() })
    }

    pub fn from_class(class: JavaClassDefinition, core: &ArmCore) -> Self {
        Self { class, core: core.clone() }
    }

    fn element_type_descriptor(&self) -> String {
        let class_name = ClassDefinition::name(&self.class);
        class_name[1..].into()
    }

    pub fn element_size(&self) -> usize {
        array_element_size(&self.element_type())
    }

    pub fn element_type(&self) -> JavaType {
        JavaType::parse(&self.element_type_descriptor())
    }
}

#[async_trait::async_trait]
impl ArrayClassDefinition for JavaArrayClassDefinition {
    fn element_type_name(&self) -> String {
        self.element_type_descriptor()
    }

    async fn instantiate_array(&self, jvm: &Jvm, length: usize) -> JvmResult<Box<dyn ClassInstance>> {
        match JavaArrayClassInstance::new(&mut self.core.clone(), self, length) {
            Ok(instance) => Ok(Box::new(instance)),
            Err(error) => Err(jvm.exception("net/wie/WieError", &format!("Failed to instantiate array: {error}")).await),
        }
    }
}

impl Debug for JavaArrayClassDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("JavaArrayClassDefinition").field("class", &self.class).finish()
    }
}
