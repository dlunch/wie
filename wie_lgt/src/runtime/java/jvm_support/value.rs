use alloc::boxed::Box;

use jvm::ClassInstance;

use wie_core_arm::ArmCore;
use wie_jvm_support::native::NativeJavaValueCodec;

use super::{ClassRegistry, JavaArrayClassInstance, JavaClassInstance};

#[derive(Clone)]
pub struct JavaValueCodec {
    core: ArmCore,
    registry: ClassRegistry,
}

impl JavaValueCodec {
    pub fn new(core: &ArmCore, registry: &ClassRegistry) -> Self {
        Self {
            core: core.clone(),
            registry: registry.clone(),
        }
    }
}

impl NativeJavaValueCodec for JavaValueCodec {
    fn object_from_raw(&self, raw: u32) -> Box<dyn ClassInstance> {
        let instance = JavaClassInstance::from_raw(raw, &self.core, &self.registry);
        if jvm::ClassDefinition::name(instance.class()).starts_with('[') {
            Box::new(JavaArrayClassInstance::from_raw(raw, &self.core, &self.registry))
        } else {
            Box::new(instance)
        }
    }

    fn object_to_raw(&self, object: &dyn ClassInstance) -> u32 {
        if let Some(instance) = object.as_any().downcast_ref::<JavaClassInstance>() {
            instance.ptr_raw
        } else {
            object.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap().class_instance.ptr_raw
        }
    }
}
