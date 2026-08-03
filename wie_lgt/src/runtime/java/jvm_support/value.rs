use alloc::boxed::Box;

use jvm::{ClassInstance, JavaType, JavaValue};

use wie_core_arm::ArmCore;

use super::{ClassRegistry, JavaArrayClassInstance, JavaClassInstance, LgtJvmWord};

pub trait JavaValueExt {
    fn from_raw(raw: LgtJvmWord, r#type: &JavaType, core: &ArmCore, registry: &ClassRegistry) -> JavaValue;
    fn from_raw64(low: LgtJvmWord, high: LgtJvmWord, r#type: &JavaType) -> JavaValue;
    fn as_raw(&self) -> LgtJvmWord;
    fn as_raw64(&self) -> (LgtJvmWord, LgtJvmWord);
}

impl JavaValueExt for JavaValue {
    fn from_raw(raw: LgtJvmWord, r#type: &JavaType, core: &ArmCore, registry: &ClassRegistry) -> JavaValue {
        match r#type {
            JavaType::Void => JavaValue::Void,
            JavaType::Boolean => JavaValue::Boolean(raw != 0),
            JavaType::Byte => JavaValue::Byte(raw as i8),
            JavaType::Short => JavaValue::Short(raw as i16),
            JavaType::Int => JavaValue::Int(raw as i32),
            JavaType::Float => JavaValue::Float(f32::from_bits(raw)),
            JavaType::Char => JavaValue::Char(raw as u16),
            JavaType::Class(_) | JavaType::Array(_) => {
                if raw == 0 {
                    JavaValue::Object(None)
                } else {
                    JavaValue::Object(Some(class_instance_from_raw(raw, core, registry)))
                }
            }
            JavaType::Long | JavaType::Double | JavaType::Method(_, _) => unreachable!(),
        }
    }

    fn from_raw64(low: LgtJvmWord, high: LgtJvmWord, r#type: &JavaType) -> JavaValue {
        match r#type {
            JavaType::Long => JavaValue::Long((((high as u64) << 32) | low as u64) as i64),
            JavaType::Double => JavaValue::Double(f64::from_bits(((high as u64) << 32) | low as u64)),
            _ => unreachable!(),
        }
    }

    fn as_raw(&self) -> LgtJvmWord {
        match self {
            JavaValue::Void => 0,
            JavaValue::Boolean(value) => *value as u32,
            JavaValue::Byte(value) => *value as u32,
            JavaValue::Short(value) => *value as u32,
            JavaValue::Int(value) => *value as u32,
            JavaValue::Float(value) => value.to_bits(),
            JavaValue::Char(value) => *value as u32,
            JavaValue::Object(Some(instance)) => class_instance_raw(&**instance),
            JavaValue::Object(None) => 0,
            JavaValue::Long(_) | JavaValue::Double(_) => unreachable!(),
        }
    }

    fn as_raw64(&self) -> (LgtJvmWord, LgtJvmWord) {
        let value = match self {
            JavaValue::Long(value) => *value as u64,
            JavaValue::Double(value) => value.to_bits(),
            _ => unreachable!(),
        };
        (value as u32, (value >> 32) as u32)
    }
}

pub fn class_instance_raw(instance: &dyn ClassInstance) -> u32 {
    if let Some(instance) = instance.as_any().downcast_ref::<JavaClassInstance>() {
        instance.ptr_raw
    } else {
        instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap().class_instance.ptr_raw
    }
}

pub fn class_instance_from_raw(ptr_raw: u32, core: &ArmCore, registry: &ClassRegistry) -> Box<dyn ClassInstance> {
    let instance = JavaClassInstance::from_raw(ptr_raw, core, registry);
    if jvm::ClassDefinition::name(instance.class()).starts_with('[') {
        Box::new(JavaArrayClassInstance::from_raw(ptr_raw, core, registry))
    } else {
        Box::new(instance)
    }
}
