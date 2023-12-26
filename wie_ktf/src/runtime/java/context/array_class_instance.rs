use alloc::{string::String, vec::Vec};

use bytemuck::{cast_slice, cast_vec, Pod};
use num_traits::FromBytes;

use jvm::{ArrayClassInstance, ClassInstance, Field, JavaValue, JvmResult};

use wie_base::util::{read_generic, write_generic, ByteRead, ByteWrite};
use wie_core_arm::ArmCore;
use wie_impl_java::{JavaResult, JavaWord};

use super::{array_class::JavaArrayClass, class_instance::JavaClassInstance};

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

    pub fn new(core: &mut ArmCore, array_class: &JavaArrayClass, count: JavaWord) -> JavaResult<Self> {
        let element_size = array_class.element_size()?;
        let class_instance = JavaClassInstance::instantiate(core, &array_class.class, count * element_size + 4)?;

        let length_address = class_instance.field_address(0)?;
        write_generic(core, length_address, count as u32)?;

        Ok(Self::from_raw(class_instance.ptr_raw, core))
    }

    pub fn load_array<T, const B: usize>(&self, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<T>>
    where
        T: FromBytes<Bytes = [u8; B]> + Pod,
    {
        let array_length = self.array_length()?;
        if offset + count > array_length {
            anyhow::bail!("Array index out of bounds");
        }

        let base_address = self.class_instance.field_address(4)?;

        let element_size = self.array_element_size()?;
        assert!(element_size == core::mem::size_of::<T>() as _, "Incorrect element size");

        let values_raw = self
            .core
            .read_bytes(base_address + (element_size * offset) as u32, (count * element_size) as _)?;
        if B != 1 {
            Ok(values_raw
                .chunks(element_size as _)
                .map(|x| T::from_le_bytes(x.try_into().unwrap()))
                .collect::<Vec<_>>())
        } else {
            Ok(cast_vec(values_raw))
        }
    }

    pub fn store_array<T>(&mut self, offset: JavaWord, values: &[T]) -> JavaResult<()>
    where
        T: Pod,
    {
        let array_length = self.array_length()?;
        if offset + values.len() as JavaWord > array_length {
            anyhow::bail!("Array index out of bounds");
        }

        let base_address = self.class_instance.field_address(4)?;

        let element_size = self.array_element_size()?;
        assert!(element_size == core::mem::size_of::<T>() as _, "Incorrect element size");

        let values_u8 = cast_slice(values);

        self.core.write_bytes(base_address + (element_size * offset) as u32, values_u8)
    }

    pub fn array_length(&self) -> JavaResult<JavaWord> {
        let length_address = self.class_instance.field_address(0)?;
        let result: u32 = read_generic(&self.core, length_address)?;

        Ok(result as _)
    }

    pub fn array_element_size(&self) -> JavaResult<JavaWord> {
        let array_class = JavaArrayClass::from_raw(self.class_instance.class()?.ptr_raw, &self.core);

        array_class.element_size()
    }
}

impl ClassInstance for JavaArrayClassInstance {
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
    fn store(&mut self, _offset: usize, _values: &[JavaValue]) -> JvmResult<()> {
        todo!()
    }

    fn load(&self, _offset: usize, _count: usize) -> JvmResult<Vec<JavaValue>> {
        todo!()
    }

    fn length(&self) -> usize {
        todo!()
    }
}
