use alloc::vec::Vec;

use bytemuck::{cast_slice, cast_vec, Pod};
use num_traits::FromBytes;

use wie_base::util::{ByteRead, ByteWrite};
use wie_impl_java::{JavaResult, JavaWord};

use super::{class::JavaClass, class_instance::JavaClassInstance, KtfJavaContext};

pub struct JavaArrayClassInstance {
    pub(crate) class_instance: JavaClassInstance,
}

impl JavaArrayClassInstance {
    pub fn from_raw(ptr_raw: u32) -> Self {
        Self {
            class_instance: JavaClassInstance::from_raw(ptr_raw),
        }
    }

    pub async fn new(context: &mut KtfJavaContext<'_>, array_class: JavaClass, count: JavaWord) -> JavaResult<Self> {
        let element_size = Self::get_array_element_size(context, &array_class)?;
        let instance = JavaClassInstance::instantiate(context, &array_class, count * element_size + 4).await?;

        instance.write_field(context, 0, count)?;

        Ok(Self { class_instance: instance })
    }

    pub fn load_array<T, const B: usize>(&self, context: &KtfJavaContext<'_>, offset: JavaWord, count: JavaWord) -> JavaResult<Vec<T>>
    where
        T: FromBytes<Bytes = [u8; B]> + Pod,
    {
        let array_length = self.array_length(context)?;
        if offset + count > array_length {
            anyhow::bail!("Array index out of bounds");
        }

        let base_address = self.class_instance.field_address(context, 4)?;

        let element_size = self.array_element_size(context)?;
        assert!(element_size == core::mem::size_of::<T>() as _, "Incorrect element size");

        let values_raw = context
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

    pub fn store_array<T>(&self, context: &mut KtfJavaContext<'_>, offset: JavaWord, values: &[T]) -> JavaResult<()>
    where
        T: Pod,
    {
        let array_length = self.array_length(context)?;
        if offset + values.len() as JavaWord > array_length {
            anyhow::bail!("Array index out of bounds");
        }

        let base_address = self.class_instance.field_address(context, 4)?;

        let element_size = self.array_element_size(context)?;
        assert!(element_size == core::mem::size_of::<T>() as _, "Incorrect element size");

        let values_u8 = cast_slice(values);

        context.core.write_bytes(base_address + (element_size * offset) as u32, values_u8)
    }

    pub fn array_length(&self, context: &KtfJavaContext<'_>) -> JavaResult<JavaWord> {
        let result = self.class_instance.read_field(context, 0)?;

        Ok(result as _)
    }

    pub fn array_element_size(&self, context: &KtfJavaContext<'_>) -> JavaResult<JavaWord> {
        let class = self.class_instance.class(context)?;

        Self::get_array_element_size(context, &class)
    }

    fn get_array_element_size(context: &KtfJavaContext<'_>, class: &JavaClass) -> JavaResult<JavaWord> {
        let class_name = class.name(context)?;

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
