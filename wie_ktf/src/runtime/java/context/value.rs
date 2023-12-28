use alloc::{boxed::Box, rc::Rc};
use core::cell::RefCell;

use jvm::{JavaType, JavaValue};

use wie_core_arm::ArmCore;

use super::{array_class_instance::JavaArrayClassInstance, class_instance::JavaClassInstance, KtfJvmWord};

pub trait JavaValueExt {
    fn from_raw(raw: KtfJvmWord, r#type: &JavaType, core: &ArmCore) -> JavaValue;
    fn as_raw(&self) -> KtfJvmWord;
}

impl JavaValueExt for JavaValue {
    fn from_raw(raw: KtfJvmWord, r#type: &JavaType, core: &ArmCore) -> JavaValue {
        match r#type {
            JavaType::Void => JavaValue::Void,
            JavaType::Boolean => JavaValue::Boolean(raw != 0),
            JavaType::Byte => JavaValue::Byte(raw as i8),
            JavaType::Short => JavaValue::Short(raw as i16),
            JavaType::Int => JavaValue::Int(raw as i32),
            JavaType::Long => JavaValue::Long(raw as i64),
            JavaType::Float => JavaValue::Float(f32::from_bits(raw)),
            JavaType::Double => JavaValue::Double(f64::from_bits(raw as u64)),
            JavaType::Char => JavaValue::Char(raw as u16),
            JavaType::Class(_) => {
                if raw != 0 {
                    let instance = JavaClassInstance::from_raw(raw, core);

                    JavaValue::Object(Some(Rc::new(RefCell::new(Box::new(instance)))))
                } else {
                    JavaValue::Object(None)
                }
            }
            JavaType::Array(_) => {
                if raw != 0 {
                    let instance = JavaArrayClassInstance::from_raw(raw, core);

                    JavaValue::Object(Some(Rc::new(RefCell::new(Box::new(instance)))))
                } else {
                    JavaValue::Object(None)
                }
            }
            _ => todo!(),
        }
    }

    fn as_raw(&self) -> KtfJvmWord {
        match self {
            JavaValue::Void => 0,
            JavaValue::Boolean(x) => *x as _,
            JavaValue::Byte(x) => *x as _,
            JavaValue::Short(x) => *x as _,
            JavaValue::Int(x) => *x as _,
            JavaValue::Long(x) => *x as _,
            JavaValue::Float(x) => x.to_bits() as _,
            JavaValue::Double(x) => x.to_bits() as _,
            JavaValue::Char(x) => *x as _,
            JavaValue::Object(x) => {
                if let Some(x) = x {
                    let instance = x.as_ref().borrow();
                    if let Some(x) = instance.as_any().downcast_ref::<JavaClassInstance>() {
                        x.ptr_raw as _
                    } else if let Some(x) = instance.as_any().downcast_ref::<JavaArrayClassInstance>() {
                        x.class_instance.ptr_raw as _
                    } else {
                        panic!("Unknown instance type")
                    }
                } else {
                    0
                }
            }
        }
    }
}
