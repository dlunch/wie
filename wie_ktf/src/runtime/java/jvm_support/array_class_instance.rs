use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
};

use jvm::{ArrayClassInstance, ArrayRawBuffer, ArrayRawBufferMut, ClassDefinition, ClassInstance, JavaType, JavaValue, Result as JvmResult};

use wie_core_arm::ArmCore;
use wie_util::{ByteRead, ByteWrite, read_generic, write_generic};

use super::{Result, array_class_definition::JavaArrayClassDefinition, class_instance::JavaClassInstance, value::JavaValueExt};

#[derive(Clone)]
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

    pub fn new(core: &mut ArmCore, array_class: &JavaArrayClassDefinition, count: usize) -> Result<Self> {
        let element_size = array_class.element_size()?;
        let class_instance = JavaClassInstance::instantiate(core, &array_class.class, count * element_size + 4)?;

        let length_address = class_instance.field_address(0)?;
        write_generic(core, length_address, count as u32)?;

        Ok(Self::from_raw(class_instance.ptr_raw, core))
    }

    pub fn load_raw(&self, offset: usize, buf: &mut [u8]) -> Result<()> {
        let base_address = self.base_address()?;

        self.core.read_bytes(base_address + offset as u32, buf)?;

        Ok(())
    }

    pub fn store_raw(&mut self, offset: usize, values_raw: Vec<u8>) -> Result<()> {
        let base_address = self.base_address()?;

        self.core.write_bytes(base_address + offset as u32, &values_raw)
    }

    pub fn array_length(&self) -> Result<usize> {
        let length_address = self.class_instance.field_address(0)?;
        let result: u32 = read_generic(&self.core, length_address)?;

        Ok(result as _)
    }

    fn element_size(&self) -> Result<usize> {
        let array_class = JavaArrayClassDefinition::from_raw(self.class_instance.class()?.ptr_raw, &self.core);

        array_class.element_size()
    }

    fn element_type(&self) -> Result<JavaType> {
        let array_class = JavaArrayClassDefinition::from_raw(self.class_instance.class()?.ptr_raw, &self.core);

        Ok(JavaType::parse(&array_class.element_type_descriptor()?))
    }

    fn base_address(&self) -> Result<u32> {
        self.class_instance.field_address(4)
    }
}

#[async_trait::async_trait]
impl ArrayClassInstance for JavaArrayClassInstance {
    fn destroy(self: Box<Self>) {
        let field_size = self.element_size().unwrap() * self.array_length().unwrap() + 4;

        self.class_instance.destroy(field_size as _).unwrap()
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(self.class_instance.class().unwrap())
    }

    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        self.class_instance.equals(other)
    }

    fn store(&mut self, offset: usize, values: Box<[JavaValue]>) -> JvmResult<()> {
        let element_size = self.element_size().unwrap();

        let values = values.to_vec();

        let raw_values = match element_size {
            1 => values.into_iter().map(|x| x.as_raw() as u8).collect::<Vec<_>>(),
            2 => values
                .into_iter()
                .map(|x| x.as_raw() as u16)
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>(),
            4 => values.into_iter().map(|x| x.as_raw()).flat_map(u32::to_le_bytes).collect::<Vec<_>>(),
            _ => unreachable!(),
        };

        let offset = offset * element_size;
        self.store_raw(offset as _, raw_values).unwrap();

        Ok(())
    }

    fn load(&self, offset: usize, count: usize) -> JvmResult<Vec<JavaValue>> {
        let element_size = self.element_size().unwrap();
        let offset = offset * element_size;

        let mut values_raw = vec![0; count * element_size];
        self.load_raw(offset as _, &mut values_raw).unwrap();

        let element_type = self.element_type().unwrap();

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
            _ => unreachable!(),
        })
    }

    fn raw_buffer(&self) -> JvmResult<Box<dyn ArrayRawBuffer>> {
        Ok(Box::new(ArrayRawBufferImpl {
            core: self.core.clone(),
            base_address: self.base_address().unwrap() as _,
            element_size: self.element_size().unwrap() as _,
        }))
    }

    fn raw_buffer_mut(&mut self) -> JvmResult<Box<dyn ArrayRawBufferMut>> {
        Ok(Box::new(ArrayRawBufferImpl {
            core: self.core.clone(),
            base_address: self.base_address().unwrap() as _,
            element_size: self.element_size().unwrap() as _,
        }))
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

impl Hash for JavaArrayClassInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.class_instance.hash(state);
    }
}

struct ArrayRawBufferImpl {
    core: ArmCore,
    base_address: u32,
    element_size: u32,
}

impl ArrayRawBuffer for ArrayRawBufferImpl {
    fn read(&self, offset: usize, buffer: &mut [u8]) -> JvmResult<()> {
        let address = self.base_address + (offset as u32) * self.element_size;
        self.core.read_bytes(address, buffer).unwrap();

        Ok(())
    }
}

impl ArrayRawBufferMut for ArrayRawBufferImpl {
    fn write(&mut self, offset: usize, buffer: &[u8]) -> JvmResult<()> {
        let address = self.base_address + (offset as u32) * self.element_size;
        self.core.write_bytes(address, buffer).unwrap();

        Ok(())
    }
}
