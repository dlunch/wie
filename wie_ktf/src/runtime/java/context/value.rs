use alloc::{boxed::Box, rc::Rc};
use core::cell::RefCell;

use jvm::{JavaType, JavaValue};

use wie_core_arm::ArmCore;
use wie_impl_java::JavaWord;

use super::{array_class_instance::JavaArrayClassInstance, class_instance::JavaClassInstance};

pub trait JavaValueExt {
    fn from_raw(raw: JavaWord, descriptor: &str, core: &ArmCore) -> JavaValue;
    fn as_raw(&self, descriptor: &str) -> JavaWord;
}

impl JavaValueExt for JavaValue {
    fn from_raw(raw: JavaWord, descriptor: &str, core: &ArmCore) -> JavaValue {
        let r#type = JavaType::parse(descriptor);

        match r#type {
            JavaType::Boolean => JavaValue::Boolean(raw != 0),
            JavaType::Byte => JavaValue::Byte(raw as i8),
            JavaType::Short => JavaValue::Short(raw as i16),
            JavaType::Int => JavaValue::Int(raw as i32),
            JavaType::Long => JavaValue::Long(raw as i64),
            JavaType::Float => JavaValue::Float(f32::from_bits(raw as u32)),
            JavaType::Double => JavaValue::Double(f64::from_bits(raw as u64)),
            JavaType::Char => JavaValue::Char(raw as u16),
            JavaType::Class(_) => {
                if raw != 0 {
                    let instance = JavaClassInstance::from_raw(raw as u32, core);

                    JavaValue::Object(Some(Rc::new(RefCell::new(Box::new(instance)))))
                } else {
                    JavaValue::Object(None)
                }
            }
            JavaType::Array(_) => {
                if raw != 0 {
                    let instance = JavaArrayClassInstance::from_raw(raw as u32, core);

                    JavaValue::Object(Some(Rc::new(RefCell::new(Box::new(instance)))))
                } else {
                    JavaValue::Object(None)
                }
            }
            _ => todo!(),
        }
    }

    fn as_raw(&self, descriptor: &str) -> JavaWord {
        let r#type = JavaType::parse(descriptor);

        match r#type {
            JavaType::Boolean => self.as_boolean() as u32 as JavaWord,
            JavaType::Byte => self.as_byte() as u32 as JavaWord,
            JavaType::Short => self.as_short() as u32 as JavaWord,
            JavaType::Int => self.as_int() as u32 as JavaWord,
            JavaType::Long => self.as_long() as JavaWord,
            JavaType::Float => self.as_float().to_bits() as JavaWord,
            JavaType::Double => self.as_double().to_bits() as JavaWord,
            JavaType::Char => self.as_char() as u32 as JavaWord,
            JavaType::Class(_) => {
                if let Some(x) = self.as_object() {
                    let instance = x.as_ref().borrow();
                    let instance = instance.as_any().downcast_ref::<JavaClassInstance>().unwrap();

                    instance.ptr_raw as _
                } else {
                    0
                }
            }
            JavaType::Array(_) => {
                if let Some(x) = self.as_object() {
                    let instance = x.as_ref().borrow();
                    let instance = instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap();

                    instance.class_instance.ptr_raw as _
                } else {
                    0
                }
            }
            _ => todo!(),
        }
    }
}
