use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
};

use jvm::{ArrayClassInstance, ArrayRawBuffer, ArrayRawBufferMut, ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Result as JvmResult};

use wie_core_arm::ArmCore;
use wie_jvm_support::native::{decode_array_values, encode_array_values};
use wie_util::{ByteRead, ByteWrite, read_generic, write_generic};

use super::{Result, array_class_definition::JavaArrayClassDefinition, class_instance::JavaClassInstance, value::JavaValueCodec};

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
impl ClassInstance for JavaArrayClassInstance {
    fn destroy(self: Box<Self>) {
        let field_size = self.element_size().unwrap() * self.array_length().unwrap() + 4;

        self.class_instance.destroy(field_size as _).unwrap()
    }

    fn identity(&self) -> usize {
        self.class_instance.ptr_raw as _
    }

    fn shallow_clone(&self) -> JvmResult<Box<dyn ClassInstance>> {
        let mut core = self.core.clone();
        let array_class = JavaArrayClassDefinition::from_raw(self.class_instance.class().unwrap().ptr_raw, &self.core);
        let length = self.array_length().unwrap();

        let mut instance = Self::new(&mut core, &array_class, length).unwrap();

        let mut buf = vec![0; length * self.element_size().unwrap()];
        self.load_raw(0, &mut buf).unwrap();
        instance.store_raw(0, buf).unwrap();

        Ok(Box::new(instance))
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(self.class_instance.class().unwrap())
    }

    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        let other = other.as_any().downcast_ref::<JavaArrayClassInstance>();
        if other.is_none() {
            return Ok(false);
        }

        Ok(self.class_instance.ptr_raw == other.unwrap().class_instance.ptr_raw)
    }

    fn as_array_instance(&self) -> Option<&dyn ArrayClassInstance> {
        Some(self)
    }

    fn as_array_instance_mut(&mut self) -> Option<&mut dyn ArrayClassInstance> {
        Some(self)
    }

    fn get_field(&self, _field: &dyn Field) -> JvmResult<JavaValue> {
        panic!("Array classes do not have fields")
    }

    fn put_field(&mut self, _field: &dyn Field, _value: JavaValue) -> JvmResult<()> {
        panic!("Array classes do not have fields")
    }
}

impl ArrayClassInstance for JavaArrayClassInstance {
    fn store(&mut self, offset: usize, values: Box<[JavaValue]>) -> JvmResult<()> {
        let element_size = self.element_size().unwrap();
        let element_type = self.element_type().unwrap();
        let raw_values = encode_array_values(&JavaValueCodec::new(&self.core), &element_type, &values);

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

        Ok(decode_array_values(&JavaValueCodec::new(&self.core), &element_type, &values_raw))
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
