use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    mem::size_of,
};

use jvm::{ArrayClassInstance, ArrayRawBuffer, ArrayRawBufferMut, ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Result as JvmResult};

use wie_core_arm::ArmCore;
use wie_jvm_support::native::{decode_array_values, encode_array_values};
use wie_util::{ByteRead, ByteWrite, Result, read_generic, write_generic};

use super::{JavaArrayClassDefinition, JavaClassInstance, value::JavaValueCodec};

#[derive(Clone)]
pub struct JavaArrayClassInstance {
    pub class_instance: JavaClassInstance,
    core: ArmCore,
}

impl JavaArrayClassInstance {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self {
            class_instance: JavaClassInstance::from_raw(ptr_raw, core),
            core: core.clone(),
        }
    }

    pub fn new(core: &mut ArmCore, class: &JavaArrayClassDefinition, length: usize) -> Result<Self> {
        let storage_size = size_of::<u32>() + length * class.element_size();
        let class_instance = JavaClassInstance::instantiate(core, &class.class, storage_size)?;
        write_generic(core, class_instance.ptr_fields()?, length as u32)?;

        Ok(Self::from_raw(class_instance.ptr_raw, core))
    }

    fn storage_size(&self) -> usize {
        size_of::<u32>() + self.array_length() * self.element_size()
    }

    fn array_length(&self) -> usize {
        let length: u32 = read_generic(&self.core, self.class_instance.ptr_fields().unwrap()).unwrap();
        length as usize
    }

    fn base_address(&self) -> u32 {
        self.class_instance.storage_address(size_of::<u32>()).unwrap()
    }

    fn element_size(&self) -> usize {
        JavaArrayClassDefinition::from_class(self.class_instance.class().unwrap(), &self.core).element_size()
    }

    fn element_type(&self) -> JavaType {
        JavaArrayClassDefinition::from_class(self.class_instance.class().unwrap(), &self.core).element_type()
    }

    fn load_raw(&self, byte_offset: usize, buffer: &mut [u8]) -> Result<()> {
        self.core.read_bytes(self.base_address() + byte_offset as u32, buffer)?;
        Ok(())
    }

    fn store_raw(&mut self, byte_offset: usize, buffer: &[u8]) -> Result<()> {
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
        let class = JavaArrayClassDefinition::from_class(self.class_instance.class().unwrap(), &self.core);
        let mut instance = Self::new(&mut core, &class, self.array_length()).unwrap();
        let mut data = vec![0; self.array_length() * self.element_size()];
        self.load_raw(0, &mut data).unwrap();
        instance.store_raw(0, &data).unwrap();
        Ok(Box::new(instance))
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(JavaArrayClassDefinition::from_class(self.class_instance.class().unwrap(), &self.core))
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
        let element_size = self.element_size();
        let bytes = encode_array_values(&JavaValueCodec::new(&self.core), &self.element_type(), &values);
        self.store_raw(offset * element_size, &bytes).unwrap();
        Ok(())
    }

    fn load(&self, offset: usize, count: usize) -> JvmResult<Vec<JavaValue>> {
        let element_size = self.element_size();
        let mut bytes = vec![0; count * element_size];
        self.load_raw(offset * element_size, &mut bytes).unwrap();
        let element_type = self.element_type();

        Ok(decode_array_values(&JavaValueCodec::new(&self.core), &element_type, &bytes))
    }

    fn raw_buffer(&self) -> JvmResult<Box<dyn ArrayRawBuffer>> {
        Ok(Box::new(ArrayRawBufferImpl {
            core: self.core.clone(),
            base_address: self.base_address(),
            element_size: self.element_size(),
        }))
    }

    fn raw_buffer_mut(&mut self) -> JvmResult<Box<dyn ArrayRawBufferMut>> {
        Ok(Box::new(ArrayRawBufferImpl {
            core: self.core.clone(),
            base_address: self.base_address(),
            element_size: self.element_size(),
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
