use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    mem::size_of,
};

use jvm::{ArrayClassInstance, ArrayRawBuffer, ArrayRawBufferMut, ClassDefinition, ClassInstance, Field, JavaValue, Result as JvmResult};

use wie_core_arm::ArmCore;
use wie_util::{ByteRead, ByteWrite, read_generic, write_generic};

use super::{ClassRegistry, JavaArrayClassDefinition, JavaClassInstance, Result, value::JavaValueExt};

#[derive(Clone)]
pub struct JavaArrayClassInstance {
    pub class_instance: JavaClassInstance,
    class: JavaArrayClassDefinition,
    core: ArmCore,
}

impl JavaArrayClassInstance {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore, registry: &ClassRegistry) -> Self {
        let class_instance = JavaClassInstance::from_raw(ptr_raw, core, registry);
        let class = JavaArrayClassDefinition::from_class(class_instance.class().clone(), core);
        Self {
            class_instance,
            class,
            core: core.clone(),
        }
    }

    pub fn new(core: &mut ArmCore, class: &JavaArrayClassDefinition, length: usize) -> Result<Self> {
        let storage_size = size_of::<u32>() + length * class.element_size();
        let class_instance = JavaClassInstance::instantiate(core, &class.class, storage_size)?;
        write_generic(core, class_instance.ptr_fields()?, length as u32)?;

        Ok(Self {
            class_instance,
            class: class.clone(),
            core: core.clone(),
        })
    }

    fn storage_size(&self) -> usize {
        size_of::<u32>() + self.array_length() * self.class.element_size()
    }

    fn array_length(&self) -> usize {
        let length: u32 = read_generic(&self.core, self.class_instance.ptr_fields().unwrap()).unwrap();
        length as usize
    }

    fn base_address(&self) -> u32 {
        self.class_instance.storage_address(size_of::<u32>()).unwrap()
    }

    pub fn load_raw(&self, byte_offset: usize, buffer: &mut [u8]) -> Result<()> {
        self.core.read_bytes(self.base_address() + byte_offset as u32, buffer)?;
        Ok(())
    }

    pub fn store_raw(&mut self, byte_offset: usize, buffer: &[u8]) -> Result<()> {
        self.core.write_bytes(self.base_address() + byte_offset as u32, buffer)
    }
}

#[async_trait::async_trait]
impl ClassInstance for JavaArrayClassInstance {
    fn destroy(self: Box<Self>) {
        let storage_size = self.storage_size();
        self.class_instance.clone().destroy_with_storage(storage_size).unwrap();
    }

    fn identity(&self) -> usize {
        self.class_instance.ptr_raw as usize
    }

    fn shallow_clone(&self) -> JvmResult<Box<dyn ClassInstance>> {
        let mut core = self.core.clone();
        let mut instance = Self::new(&mut core, &self.class, self.array_length()).unwrap();
        let mut data = vec![0; self.array_length() * self.class.element_size()];
        self.load_raw(0, &mut data).unwrap();
        instance.store_raw(0, &data).unwrap();
        Ok(Box::new(instance))
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(self.class.clone())
    }

    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        Ok(other
            .as_any()
            .downcast_ref::<JavaArrayClassInstance>()
            .is_some_and(|other| self.class_instance.ptr_raw == other.class_instance.ptr_raw))
    }

    fn as_array_instance(&self) -> Option<&dyn ArrayClassInstance> {
        Some(self)
    }

    fn as_array_instance_mut(&mut self) -> Option<&mut dyn ArrayClassInstance> {
        Some(self)
    }

    fn get_field(&self, _field: &dyn Field) -> JvmResult<JavaValue> {
        unreachable!()
    }

    fn put_field(&mut self, _field: &dyn Field, _value: JavaValue) -> JvmResult<()> {
        unreachable!()
    }
}

impl ArrayClassInstance for JavaArrayClassInstance {
    fn store(&mut self, offset: usize, values: Box<[JavaValue]>) -> JvmResult<()> {
        let element_size = self.class.element_size();
        let bytes = match element_size {
            1 => values.iter().map(JavaValueExt::as_raw).map(|x| x as u8).collect::<Vec<_>>(),
            2 => values
                .iter()
                .map(JavaValueExt::as_raw)
                .flat_map(|x| (x as u16).to_le_bytes())
                .collect::<Vec<_>>(),
            4 => values.iter().map(JavaValueExt::as_raw).flat_map(u32::to_le_bytes).collect::<Vec<_>>(),
            8 => values
                .iter()
                .flat_map(|value| {
                    let (low, high) = value.as_raw64();
                    (((high as u64) << 32) | low as u64).to_le_bytes()
                })
                .collect::<Vec<_>>(),
            _ => unreachable!(),
        };
        self.store_raw(offset * element_size, &bytes).unwrap();
        Ok(())
    }

    fn load(&self, offset: usize, count: usize) -> JvmResult<Vec<JavaValue>> {
        let element_size = self.class.element_size();
        let mut bytes = vec![0; count * element_size];
        self.load_raw(offset * element_size, &mut bytes).unwrap();
        let element_type = self.class.element_type();

        Ok(match element_size {
            1 => bytes
                .into_iter()
                .map(|value| JavaValue::from_raw(value as u32, &element_type, &self.core, self.class.class.registry()))
                .collect(),
            2 => bytes
                .chunks_exact(2)
                .map(|value| {
                    JavaValue::from_raw(
                        u16::from_le_bytes(value.try_into().unwrap()) as u32,
                        &element_type,
                        &self.core,
                        self.class.class.registry(),
                    )
                })
                .collect(),
            4 => bytes
                .chunks_exact(4)
                .map(|value| {
                    JavaValue::from_raw(
                        u32::from_le_bytes(value.try_into().unwrap()),
                        &element_type,
                        &self.core,
                        self.class.class.registry(),
                    )
                })
                .collect(),
            8 => bytes
                .chunks_exact(8)
                .map(|value| {
                    let value = u64::from_le_bytes(value.try_into().unwrap());
                    JavaValue::from_raw64(value as u32, (value >> 32) as u32, &element_type)
                })
                .collect(),
            _ => unreachable!(),
        })
    }

    fn raw_buffer(&self) -> JvmResult<Box<dyn ArrayRawBuffer>> {
        Ok(Box::new(ArrayRawBufferImpl {
            core: self.core.clone(),
            base_address: self.base_address(),
            element_size: self.class.element_size(),
        }))
    }

    fn raw_buffer_mut(&mut self) -> JvmResult<Box<dyn ArrayRawBufferMut>> {
        Ok(Box::new(ArrayRawBufferImpl {
            core: self.core.clone(),
            base_address: self.base_address(),
            element_size: self.class.element_size(),
        }))
    }

    fn length(&self) -> usize {
        self.array_length()
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
    element_size: usize,
}

impl ArrayRawBuffer for ArrayRawBufferImpl {
    fn read(&self, offset: usize, buffer: &mut [u8]) -> JvmResult<()> {
        self.core
            .read_bytes(self.base_address + (offset * self.element_size) as u32, buffer)
            .unwrap();
        Ok(())
    }
}

impl ArrayRawBufferMut for ArrayRawBufferImpl {
    fn write(&mut self, offset: usize, buffer: &[u8]) -> JvmResult<()> {
        self.core
            .write_bytes(self.base_address + (offset * self.element_size) as u32, buffer)
            .unwrap();
        Ok(())
    }
}
