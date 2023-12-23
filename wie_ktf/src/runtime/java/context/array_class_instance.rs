use alloc::vec::Vec;

use bytemuck::{cast_slice, cast_vec, Pod};
use num_traits::FromBytes;

use wie_base::util::{read_generic, write_generic, ByteRead, ByteWrite};
use wie_core_arm::ArmCore;
use wie_impl_java::{JavaResult, JavaWord};

use super::{class::JavaClass, class_instance::JavaClassInstance, KtfJavaContext};

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

    pub async fn new(context: &mut KtfJavaContext<'_>, array_class: JavaClass, count: JavaWord) -> JavaResult<Self> {
        let element_size = Self::get_array_element_size(&array_class)?;
        let class_instance: JavaClassInstance = JavaClassInstance::instantiate(context, &array_class, count * element_size + 4).await?;

        let length_address = class_instance.field_address(0)?;
        write_generic(context.core, length_address, count as u32)?;

        Ok(Self::from_raw(class_instance.ptr_raw, context.core))
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
        let class = self.class_instance.class()?;

        Self::get_array_element_size(&class)
    }

    fn get_array_element_size(class: &JavaClass) -> JavaResult<JavaWord> {
        let class_name = class.name()?;

        assert!(class_name.starts_with('['), "Not an array class {}", class_name);

        if class_name.starts_with("[L") || class_name.starts_with("[[") {
            Ok(4)
        } else {
            let element = class_name.as_bytes()[1];
            Ok(match element {
                b'B' => 1,
                b'C' => 2,
                b'I' => 4,
                b'Z' => 1,
                b'S' => 2,
                b'J' => 8,
                _ => unimplemented!("get_array_element_size {}", class_name),
            })
        }
    }
}
