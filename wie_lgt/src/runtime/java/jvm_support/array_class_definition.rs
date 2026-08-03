use alloc::{boxed::Box, format, string::String};
use core::fmt::{self, Debug, Formatter};

use jvm::{ArrayClassDefinition, ClassInstance, JavaType, Jvm, Result as JvmResult};

use wie_core_arm::ArmCore;

use super::{ClassRegistry, JavaArrayClassInstance, JavaClassDefinition, Result};

#[derive(Clone)]
pub struct JavaArrayClassDefinition {
    pub class: JavaClassDefinition,
    element_type_name: String,
    core: ArmCore,
}

impl JavaArrayClassDefinition {
    pub async fn new(core: &mut ArmCore, jvm: &Jvm, element_type_name: &str, registry: ClassRegistry) -> Result<Self> {
        let class = JavaClassDefinition::new_array(core, jvm, &format!("[{element_type_name}"), registry).await?;
        Ok(Self {
            class,
            element_type_name: element_type_name.into(),
            core: core.clone(),
        })
    }

    pub fn from_class(class: JavaClassDefinition, core: &ArmCore) -> Self {
        let name = jvm::ClassDefinition::name(&class);
        Self {
            class,
            element_type_name: name[1..].into(),
            core: core.clone(),
        }
    }

    pub fn element_size(&self) -> usize {
        match JavaType::parse(&self.element_type_name) {
            JavaType::Boolean | JavaType::Byte => 1,
            JavaType::Char | JavaType::Short => 2,
            JavaType::Int | JavaType::Float | JavaType::Class(_) | JavaType::Array(_) => 4,
            JavaType::Long | JavaType::Double => 8,
            JavaType::Void | JavaType::Method(_, _) => unreachable!(),
        }
    }

    pub fn element_type(&self) -> JavaType {
        JavaType::parse(&self.element_type_name)
    }
}

#[async_trait::async_trait]
impl ArrayClassDefinition for JavaArrayClassDefinition {
    fn element_type_name(&self) -> String {
        self.element_type_name.clone()
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
        f.debug_struct("JavaArrayClassDefinition")
            .field("element_type_name", &self.element_type_name)
            .finish()
    }
}
