use alloc::{boxed::Box, string::String, vec::Vec};
use core::fmt::{self, Debug, Formatter};

use bytemuck::cast_vec;

use jvm::{ArrayClassInstance, ClassInstance, Field, JavaType, JavaValue, JvmResult};

use wie_base::util::{read_generic, write_generic, ByteRead, ByteWrite};
use wie_core_arm::ArmCore;
use wie_impl_java::JavaResult;

use super::{array_class::JavaArrayClass, class_instance::JavaClassInstance, value::JavaValueExt};

pub struct JavaArrayClassInstance {
    pub(crate) class_instance: JavaClassInstance,
    core: ArmCore,
}

impl JavaArrayClassInstance {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self {
            class_instance: JavaClassInstance::from_raw(ptr_raw, core),
            core: core.clone(),
        }
    }

    pub fn new(core: &mut ArmCore, array_class: &JavaArrayClass, count: usize) -> JavaResult<Self> {
        let element_size = array_class.element_size()?;
        let class_instance = JavaClassInstance::instantiate(core, &array_class.class, count * element_size + 4)?;

        let length_address = class_instance.field_address(0)?;
        write_generic(core, length_address, count as u32)?;

        Ok(Self::from_raw(class_instance.ptr_raw, core))
    }

    pub fn load_array(&self, offset: usize, count: usize) -> JavaResult<Vec<u8>> {
        let array_length = self.array_length()?;
        if offset + count > array_length {
            anyhow::bail!("Array index out of bounds");
        }

        let base_address = self.class_instance.field_address(4)?;
        let element_size = self.element_size()?;

        let values_raw = self
            .core
            .read_bytes(base_address + (element_size * offset) as u32, (count * element_size) as _)?;

        Ok(values_raw)
    }

    pub fn store_array(&mut self, offset: usize, count: usize, values_raw: Vec<u8>) -> JavaResult<()> {
        let array_length = self.array_length()?;
        if offset + count > array_length {
            anyhow::bail!("Array index out of bounds");
        }

        let base_address = self.class_instance.field_address(4)?;
        let element_size = self.element_size()?;

        self.core.write_bytes(base_address + (element_size * offset) as u32, &values_raw)
    }

    pub fn array_length(&self) -> JavaResult<usize> {
        let length_address = self.class_instance.field_address(0)?;
        let result: u32 = read_generic(&self.core, length_address)?;

        Ok(result as _)
    }

    fn element_size(&self) -> JavaResult<usize> {
        let array_class = JavaArrayClass::from_raw(self.class_instance.class()?.ptr_raw, &self.core);

        array_class.element_size()
    }

    fn element_type(&self) -> JavaResult<JavaType> {
        let array_class = JavaArrayClass::from_raw(self.class_instance.class()?.ptr_raw, &self.core);

        Ok(JavaType::parse(&array_class.element_type_descriptor()?))
    }
}

impl ClassInstance for JavaArrayClassInstance {
    fn destroy(self: Box<Self>) {
        self.class_instance.destroy().unwrap()
    }

    fn class_name(&self) -> String {
        self.class_instance.class_name()
    }

    fn get_field(&self, _field: &dyn Field) -> JvmResult<JavaValue> {
        panic!("Array class instance does not have fields")
    }

    fn put_field(&mut self, _field: &dyn Field, _value: JavaValue) -> JvmResult<()> {
        panic!("Array class instance does not have fields")
    }

    fn as_array_instance(&self) -> Option<&dyn ArrayClassInstance> {
        Some(self)
    }

    fn as_array_instance_mut(&mut self) -> Option<&mut dyn ArrayClassInstance> {
        Some(self)
    }
}

impl ArrayClassInstance for JavaArrayClassInstance {
    fn store(&mut self, offset: usize, values: Box<[JavaValue]>) -> JvmResult<()> {
        let element_size = self.element_size()?;

        let values = values.to_vec();
        let count = values.len();

        let raw_values = match element_size {
            1 => values.into_iter().map(|x| x.as_raw() as u8).collect::<Vec<_>>(),
            2 => values
                .into_iter()
                .map(|x| x.as_raw() as u16)
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>(),
            4 => values.into_iter().map(|x| x.as_raw()).flat_map(u32::to_le_bytes).collect::<Vec<_>>(),
            _ => todo!(),
        };

        self.store_array(offset as _, count, raw_values)
    }

    fn load(&self, offset: usize, count: usize) -> JvmResult<Vec<JavaValue>> {
        let values_raw = self.load_array(offset as _, count as _)?;

        let element_type = self.element_type()?;
        let element_size = self.element_size()?;

        Ok(match element_size {
            1 => values_raw
                .into_iter()
                .map(|x| JavaValue::from_raw(x as _, &element_type, &self.core))
                .collect::<Vec<_>>(),
            2 => values_raw
                .chunks(2)
                .map(|x| JavaValue::from_raw(u16::from_le_bytes(x.try_into().unwrap()) as _, &element_type, &self.core))
                .collect::<Vec<_>>(),
            4 => values_raw
                .chunks(4)
                .map(|x| JavaValue::from_raw(u32::from_le_bytes(x.try_into().unwrap()) as _, &element_type, &self.core))
                .collect::<Vec<_>>(),
            _ => todo!(),
        })
    }

    fn store_bytes(&mut self, offset: usize, values: Box<[i8]>) -> JvmResult<()> {
        let values = values.to_vec();
        let count = values.len();

        self.store_array(offset as _, count, cast_vec(values))
    }

    fn load_bytes(&self, offset: usize, count: usize) -> JvmResult<Vec<i8>> {
        let values_raw = self.load_array(offset as _, count as _)?;

        Ok(cast_vec(values_raw))
    }

    fn length(&self) -> usize {
        self.array_length().unwrap()
    }
}

impl Debug for JavaArrayClassInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.class_instance.ptr_raw)
    }
}
