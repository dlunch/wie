use alloc::{boxed::Box, vec};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    mem::size_of,
};

use java_constants::FieldAccessFlags;
use jvm::{ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Result as JvmResult};
use wipi_types::lgt::java::LgtJavaClassInstance as RawJavaClassInstance;

use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::native::NativeJavaValueCodec;
use wie_util::{ByteRead, ByteWrite, Result, read_generic, write_generic};

use super::{JavaClassDefinition, JavaField, JavaReferenceField, LgtJvmWord, value::JavaValueCodec};

#[derive(Clone)]
pub struct JavaClassInstance {
    pub ptr_raw: u32,
    core: ArmCore,
}

impl JavaClassInstance {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn new(core: &mut ArmCore, class: &JavaClassDefinition) -> Result<Self> {
        Self::instantiate(core, class, class.instance_field_word_count()? * size_of::<LgtJvmWord>())
    }

    pub fn instantiate(core: &mut ArmCore, class: &JavaClassDefinition, storage_size: usize) -> Result<Self> {
        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClassInstance>() as u32)?;
        let allocated_storage_size = storage_size.max(size_of::<LgtJvmWord>());
        let ptr_fields = Allocator::alloc(core, allocated_storage_size as u32)?;
        core.write_bytes(ptr_fields, &vec![0; allocated_storage_size])?;

        write_generic(
            core,
            ptr_raw,
            RawJavaClassInstance {
                ptr_dispatch_table: class.ptr_vtable()?,
                unk1: 0,
                ptr_fields,
            },
        )?;

        Ok(Self::from_raw(ptr_raw, core))
    }

    pub fn destroy_with_storage(mut self, storage_size: usize) -> Result<()> {
        let ptr_fields = self.ptr_fields()?;
        Allocator::free(&mut self.core, ptr_fields, storage_size.max(size_of::<LgtJvmWord>()) as u32)?;
        Allocator::free(&mut self.core, self.ptr_raw, size_of::<RawJavaClassInstance>() as u32)
    }

    pub fn class(&self) -> Result<JavaClassDefinition> {
        let raw: RawJavaClassInstance = read_generic(&self.core, self.ptr_raw)?;
        let ptr_class = read_generic(&self.core, raw.ptr_dispatch_table)?;
        Ok(JavaClassDefinition::from_raw(ptr_class, &self.core))
    }

    pub fn ptr_fields(&self) -> Result<u32> {
        let raw: RawJavaClassInstance = read_generic(&self.core, self.ptr_raw)?;
        Ok(raw.ptr_fields)
    }

    pub fn storage_address(&self, byte_offset: usize) -> Result<u32> {
        Ok(self.ptr_fields()? + byte_offset as u32)
    }

    pub fn storage_size(&self) -> Result<usize> {
        Ok(self.class()?.instance_field_word_count()? * size_of::<LgtJvmWord>())
    }

    fn field_address(&self, word_index: u32) -> Result<u32> {
        self.storage_address(word_index as usize * size_of::<LgtJvmWord>())
    }
}

#[async_trait::async_trait]
impl ClassInstance for JavaClassInstance {
    fn destroy(self: Box<Self>) {
        let storage_size = self.storage_size().unwrap();
        (*self).destroy_with_storage(storage_size).unwrap();
    }

    fn identity(&self) -> usize {
        self.ptr_raw as usize
    }

    fn shallow_clone(&self) -> JvmResult<Box<dyn ClassInstance>> {
        let class = self.class().unwrap();
        let storage_size = self.storage_size().unwrap();
        let mut core = self.core.clone();
        let instance = Self::instantiate(&mut core, &class, storage_size).unwrap();
        let mut fields = vec![0; storage_size];
        if storage_size != 0 {
            core.read_bytes(self.ptr_fields().unwrap(), &mut fields).unwrap();
            core.write_bytes(instance.ptr_fields().unwrap(), &fields).unwrap();
        }
        Ok(Box::new(instance))
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(self.class().unwrap())
    }

    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        Ok(other
            .as_any()
            .downcast_ref::<JavaClassInstance>()
            .is_some_and(|other| self.ptr_raw == other.ptr_raw))
    }

    fn get_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        debug_assert!(!field.access_flags().contains(FieldAccessFlags::STATIC));
        let field_type = JavaType::parse(&field.descriptor());
        let word_index = if let Some(field) = field.as_any().downcast_ref::<JavaField>() {
            field.word_index().unwrap()
        } else {
            field.as_any().downcast_ref::<JavaReferenceField>().unwrap().word_index
        };
        let address = self.field_address(word_index).unwrap();
        let low = read_generic(&self.core, address).unwrap();
        let codec = JavaValueCodec::new(&self.core);

        Ok(if matches!(field_type, JavaType::Long | JavaType::Double) {
            let high = read_generic(&self.core, address + 4).unwrap();
            codec.decode_wide(low, high, &field_type)
        } else {
            codec.decode_word(low, &field_type)
        })
    }

    fn put_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        debug_assert!(!field.access_flags().contains(FieldAccessFlags::STATIC));
        let word_index = if let Some(field) = field.as_any().downcast_ref::<JavaField>() {
            field.word_index().unwrap()
        } else {
            field.as_any().downcast_ref::<JavaReferenceField>().unwrap().word_index
        };
        let address = self.field_address(word_index).unwrap();
        let codec = JavaValueCodec::new(&self.core);

        if matches!(value, JavaValue::Long(_) | JavaValue::Double(_)) {
            let (low, high) = codec.encode_wide(&value);
            write_generic(&mut self.core, address, low).unwrap();
            write_generic(&mut self.core, address + 4, high).unwrap();
        } else {
            write_generic(&mut self.core, address, codec.encode_word(&value)).unwrap();
        }
        Ok(())
    }
}

impl Debug for JavaClassInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.ptr_raw)
    }
}

impl Hash for JavaClassInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr_raw.hash(state);
    }
}
