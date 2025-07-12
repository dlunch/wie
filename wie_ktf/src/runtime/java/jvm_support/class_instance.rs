use alloc::{boxed::Box, vec::Vec};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    iter,
    mem::size_of,
};
use java_constants::FieldAccessFlags;

use jvm::{ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Result as JvmResult};
use wipi_types::ktf::java::JavaClassInstance as RawJavaClassInstance;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{ByteWrite, read_generic, write_generic};

use crate::runtime::java::jvm_support::KtfJvmSupport;

use super::{KtfJvmWord, Result, class_definition::JavaClassDefinition, field::JavaField, value::JavaValueExt};

#[derive(Clone)]
pub struct JavaClassInstance {
    pub(crate) ptr_raw: u32,
    core: ArmCore,
}

impl JavaClassInstance {
    pub fn from_raw(ptr_raw: u32, core: &ArmCore) -> Self {
        Self { ptr_raw, core: core.clone() }
    }

    pub fn new(core: &mut ArmCore, class: &JavaClassDefinition) -> Result<Self> {
        let field_size = class.field_size()?;

        let instance = Self::instantiate(core, class, field_size)?;

        tracing::trace!("Instantiated {} at {:#x}", class.name()?, instance.ptr_raw);

        Ok(instance)
    }

    pub fn destroy(mut self, field_size: KtfJvmWord) -> Result<()> {
        let raw = self.read_raw()?;

        Allocator::free(&mut self.core, raw.ptr_fields, (field_size + 4) as _)?;
        Allocator::free(&mut self.core, self.ptr_raw, size_of::<RawJavaClassInstance>() as _)?;

        Ok(())
    }

    pub fn class(&self) -> Result<JavaClassDefinition> {
        let raw = self.read_raw()?;

        Ok(JavaClassDefinition::from_raw(raw.ptr_class, &self.core))
    }

    pub(super) fn field_address(&self, offset: u32) -> Result<u32> {
        let raw = self.read_raw()?;

        Ok(raw.ptr_fields + offset + 4)
    }

    pub(super) fn instantiate(core: &mut ArmCore, class: &JavaClassDefinition, field_size: usize) -> Result<Self> {
        let ptr_raw = Allocator::alloc(core, size_of::<RawJavaClassInstance>() as _)?;
        let ptr_fields = Allocator::alloc(core, (field_size + 4) as _)?;

        let zero = iter::repeat_n(0, (field_size + 4) as _).collect::<Vec<_>>();
        core.write_bytes(ptr_fields, &zero)?;

        let vtable_index = KtfJvmSupport::get_vtable_index(core, class)?;

        write_generic(
            core,
            ptr_raw,
            RawJavaClassInstance {
                ptr_fields,
                ptr_class: class.ptr_raw,
            },
        )?;
        write_generic(core, ptr_fields, (vtable_index * 4) << 5)?;

        tracing::trace!("Instantiate {}, vtable_index {:#x} at {:#x}", class.name()?, vtable_index, ptr_raw);

        Ok(Self::from_raw(ptr_raw, core))
    }

    fn read_raw(&self) -> Result<RawJavaClassInstance> {
        let instance: RawJavaClassInstance = read_generic(&self.core, self.ptr_raw as _)?;

        Ok(instance)
    }
}

#[async_trait::async_trait]
impl ClassInstance for JavaClassInstance {
    fn destroy(self: Box<Self>) {
        let field_size = self.class().unwrap().field_size().unwrap();

        (*self).destroy(field_size as _).unwrap()
    }

    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(self.class().unwrap())
    }

    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        let other = other.as_any().downcast_ref::<JavaClassInstance>();
        if other.is_none() {
            return Ok(false);
        }

        Ok(self.ptr_raw == other.unwrap().ptr_raw)
    }

    fn get_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let field_type = JavaType::parse(&field.descriptor());

        assert!(!field.access_flags().contains(FieldAccessFlags::STATIC));

        let offset = field.offset().unwrap();
        let address = self.field_address(offset).unwrap();

        if matches!(field_type, JavaType::Long | JavaType::Double) {
            let value: KtfJvmWord = read_generic(&self.core, address).unwrap();
            let value_high: KtfJvmWord = read_generic(&self.core, address + 4).unwrap();

            let r#type = JavaType::parse(&field.descriptor());
            Ok(JavaValue::from_raw64(value, value_high, &r#type))
        } else {
            let value: KtfJvmWord = read_generic(&self.core, address).unwrap();

            let r#type = JavaType::parse(&field.descriptor());
            Ok(JavaValue::from_raw(value, &r#type, &self.core))
        }
    }

    fn put_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        let field = field.as_any().downcast_ref::<JavaField>().unwrap();
        let field_type = JavaType::parse(&field.descriptor());

        assert!(!field.access_flags().contains(FieldAccessFlags::STATIC));

        let offset = field.offset().unwrap();
        let address = self.field_address(offset).unwrap();

        if matches!(field_type, JavaType::Long | JavaType::Double) {
            let (value, value_high) = value.as_raw64();

            write_generic(&mut self.core, address, value).unwrap();
            write_generic(&mut self.core, address + 4, value_high).unwrap();
        } else {
            write_generic(&mut self.core, address, value.as_raw()).unwrap();
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
        self.ptr_raw.hash(state)
    }
}
