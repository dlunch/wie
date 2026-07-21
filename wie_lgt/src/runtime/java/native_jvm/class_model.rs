//! Pure-Rust JVM class model for the app's native classes: `LgtClassDefinition`
//! / `LgtClassInstance` / `LgtField` / `LgtMethod` and their metadata, plus the
//! guest `char[]` marshalling helpers.

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
};

use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassDefinition, ClassInstance, Field, JavaType, JavaValue, Jvm, Method, Result as JvmResult};
use spin::Mutex;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{Result, write_generic};

use super::shared::LgtJvmShared;
use super::{FIELD_ARRAY_WORDS, OBJ_HEADER_SIZE, OBJ_PTR_FIELDS_OFFSET};
use crate::runtime::java::native_class::LgtNativeClass;

#[derive(Clone, Debug)]
struct MethodMeta {
    name: String,
    descriptor: String,
    access_flags: MethodAccessFlags,
    code_ptr: u32,
}

#[derive(Clone, Debug)]
struct FieldMeta {
    name: String,
    descriptor: String,
    access_flags: FieldAccessFlags,
}

#[derive(Clone)]
pub struct LgtClassDefinition {
    inner: Arc<ClassInner>,
}

struct ClassInner {
    name: String,
    super_name: Option<String>,
    methods: Vec<MethodMeta>,
    fields: Vec<FieldMeta>,
    statics: Mutex<BTreeMap<String, JavaValue>>,
    core: ArmCore,
    shared: LgtJvmShared,
}

impl LgtClassDefinition {
    pub(super) fn from_native(class: &LgtNativeClass, core: ArmCore, shared: LgtJvmShared) -> Self {
        let methods = class
            .methods
            .iter()
            .map(|m| MethodMeta {
                name: m.name.clone(),
                descriptor: m.signature.clone(),
                access_flags: MethodAccessFlags::from_bits_truncate(m.access_flags as u16),
                code_ptr: m.code_ptr,
            })
            .collect();
        let fields = class
            .fields
            .iter()
            .map(|f| FieldMeta {
                name: f.name.clone(),
                descriptor: f.type_descriptor.clone(),
                access_flags: FieldAccessFlags::from_bits_truncate(f.access_flags as u16),
            })
            .collect();

        Self {
            inner: Arc::new(ClassInner {
                name: class.name.clone(),
                super_name: class.parent_name.clone(),
                methods,
                fields,
                statics: Mutex::new(BTreeMap::new()),
                core,
                shared,
            }),
        }
    }
}

impl LgtClassDefinition {
    /// Native code pointer of a method declared on this class, by `(name, descriptor)`.
    /// Returns `None` if the class doesn't declare it (e.g. it's inherited from a
    /// platform ancestor). Used to resolve the card lifecycle methods' addresses from
    /// app metadata instead of hardcoding them.
    pub(super) fn method_code_ptr(&self, name: &str, descriptor: &str) -> Option<u32> {
        self.inner
            .methods
            .iter()
            .find(|m| m.name == name && m.descriptor == descriptor)
            .map(|m| m.code_ptr)
    }
}

impl Debug for LgtClassDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "LgtClassDefinition({})", self.inner.name)
    }
}

#[async_trait::async_trait]
impl ClassDefinition for LgtClassDefinition {
    fn name(&self) -> String {
        self.inner.name.clone()
    }
    fn super_class_name(&self) -> Option<String> {
        self.inner.super_name.clone()
    }
    fn interface_names(&self) -> Vec<String> {
        Vec::new() // LGT native classes don't track declared interfaces
    }
    fn access_flags(&self) -> ClassAccessFlags {
        ClassAccessFlags::PUBLIC
    }

    async fn instantiate(&self, jvm: &Jvm) -> JvmResult<Box<dyn ClassInstance>> {
        let mut core = self.inner.core.clone();

        let vtable_word = self.inner.shared.vtable_word();
        let alloc = (|| -> Result<u32> {
            let ptr_fields = Allocator::alloc(&mut core, FIELD_ARRAY_WORDS * 4)?;
            wie_util::ByteWrite::write_bytes(&mut core, ptr_fields, &[0u8; (FIELD_ARRAY_WORDS * 4) as usize])?;
            let ptr_raw = Allocator::alloc(&mut core, OBJ_HEADER_SIZE)?;
            write_generic(&mut core, ptr_raw, vtable_word)?; // +0: virtual-method table base
            write_generic(&mut core, ptr_raw + 4, 0u32)?;
            write_generic(&mut core, ptr_raw + OBJ_PTR_FIELDS_OFFSET, ptr_fields)?;
            Ok(ptr_raw)
        })();
        let ptr_raw = match alloc {
            Ok(p) => p,
            Err(e) => return Err(jvm.exception("java/lang/OutOfMemoryError", &e.to_string()).await),
        };

        let instance = LgtClassInstance {
            guest_ptr: ptr_raw,
            definition: self.clone(),
            jvm_fields: Arc::new(Mutex::new(BTreeMap::new())),
        };
        self.inner.shared.register_instance(ptr_raw, Box::new(instance.clone()));

        tracing::trace!("LGT instantiate {} -> guest {ptr_raw:#x}", self.inner.name);
        Ok(Box::new(instance))
    }

    async fn prepare(&self, _: &Jvm) -> JvmResult<()> {
        Ok(()) // no constant-pool static initialisers to materialise for LGT native classes
    }

    fn method(&self, name: &str, descriptor: &str, _is_static: bool) -> Option<Box<dyn Method>> {
        self.inner.methods.iter().find(|m| m.name == name && m.descriptor == descriptor).map(|m| {
            Box::new(LgtMethod {
                class_name: self.inner.name.clone(),
                meta: m.clone(),
                core: self.inner.core.clone(),
                shared: self.inner.shared.clone(),
            }) as Box<dyn Method>
        })
    }

    fn field(&self, name: &str, descriptor: &str, _is_static: bool) -> Option<Box<dyn Field>> {
        self.inner
            .fields
            .iter()
            .find(|f| f.name == name && f.descriptor == descriptor)
            .map(|f| Box::new(LgtField { meta: f.clone() }) as Box<dyn Field>)
    }

    fn fields(&self) -> Vec<Box<dyn Field>> {
        self.inner
            .fields
            .iter()
            .map(|f| Box::new(LgtField { meta: f.clone() }) as Box<dyn Field>)
            .collect()
    }

    fn get_static_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let key = field_key(&field.name(), &field.descriptor());
        Ok(self
            .inner
            .statics
            .lock()
            .get(&key)
            .cloned()
            .unwrap_or_else(|| JavaType::parse(&field.descriptor()).default()))
    }
    fn put_static_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        self.inner.statics.lock().insert(field_key(&field.name(), &field.descriptor()), value);
        Ok(())
    }
}

// ---- instance ----

#[derive(Clone)]
pub struct LgtClassInstance {
    pub(super) guest_ptr: u32,
    pub(super) definition: LgtClassDefinition,
    // JVM-side field storage. Not yet unified with the guest field array at
    // `guest_ptr` (see `docs/lgt_abi.md` §5): fields written by ARM code and via the
    // JVM currently live in separate stores. For the current reach (boot + setup) the
    // two paths don't alias the same field, so this hasn't surfaced.
    pub(super) jvm_fields: Arc<Mutex<BTreeMap<String, JavaValue>>>,
}

impl Debug for LgtClassInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{:#x}", self.definition.inner.name, self.guest_ptr)
    }
}
impl Hash for LgtClassInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.guest_ptr.hash(state)
    }
}

#[async_trait::async_trait]
impl ClassInstance for LgtClassInstance {
    fn destroy(self: Box<Self>) {}
    fn class_definition(&self) -> Box<dyn ClassDefinition> {
        Box::new(self.definition.clone())
    }
    fn equals(&self, other: &dyn ClassInstance) -> JvmResult<bool> {
        Ok(other
            .as_any()
            .downcast_ref::<LgtClassInstance>()
            .map(|o| o.guest_ptr == self.guest_ptr)
            .unwrap_or(false))
    }
    fn get_field(&self, field: &dyn Field) -> JvmResult<JavaValue> {
        let key = field_key(&field.name(), &field.descriptor());
        Ok(self
            .jvm_fields
            .lock()
            .get(&key)
            .cloned()
            .unwrap_or_else(|| JavaType::parse(&field.descriptor()).default()))
    }
    fn put_field(&mut self, field: &dyn Field, value: JavaValue) -> JvmResult<()> {
        self.jvm_fields.lock().insert(field_key(&field.name(), &field.descriptor()), value);
        Ok(())
    }
}

// ---- field / method ----

#[derive(Debug)]
struct LgtField {
    meta: FieldMeta,
}
impl Field for LgtField {
    fn name(&self) -> String {
        self.meta.name.clone()
    }
    fn descriptor(&self) -> String {
        self.meta.descriptor.clone()
    }
    fn access_flags(&self) -> FieldAccessFlags {
        self.meta.access_flags
    }
}

struct LgtMethod {
    class_name: String,
    meta: MethodMeta,
    core: ArmCore,
    shared: LgtJvmShared,
}
impl Debug for LgtMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "LgtMethod({}.{}{})", self.class_name, self.meta.name, self.meta.descriptor)
    }
}

#[async_trait::async_trait]
impl Method for LgtMethod {
    fn name(&self) -> String {
        self.meta.name.clone()
    }
    fn descriptor(&self) -> String {
        self.meta.descriptor.clone()
    }
    fn access_flags(&self) -> MethodAccessFlags {
        self.meta.access_flags
    }

    async fn run(&self, _jvm: &Jvm, args: Box<[JavaValue]>) -> JvmResult<JavaValue> {
        let mut core = self.core.clone();
        let params: Vec<u32> = args.iter().map(|v| self.shared.value_to_guest(&mut core, v)).collect();

        tracing::debug!(
            "LGT dispatch -> native {}.{}{} code={:#x} params={:x?}",
            self.class_name,
            self.meta.name,
            self.meta.descriptor,
            self.meta.code_ptr,
            params
        );
        // cp39: drive the per-frame card step just before the card's paint, mirroring
        // ez-i's per-frame tick (advance state machine, then paint). Without it `o.paint`
        // early-returns forever (the `o.g` gate stays 0). See `Self::drive_card_step`.
        if self.class_name == "o" && self.meta.name == "paint" {
            let card_this = params.first().copied().unwrap_or(0);
            self.shared.drive_card_step(&mut core, card_this).await;
        }
        let r0: u32 = match core.run_function(self.meta.code_ptr, &params).await {
            Ok(r) => r,
            Err(e) => {
                let msg = format!(
                    "native dispatch {}.{}{} @ {:#x}: {e}",
                    self.class_name, self.meta.name, self.meta.descriptor, self.meta.code_ptr
                );
                return Err(self.shared.jvm.exception("java/lang/Error", &msg).await);
            }
        };

        let ret = match JavaType::parse(&self.meta.descriptor) {
            JavaType::Method(_, ret) => *ret,
            _ => JavaType::Void,
        };
        Ok(self.shared.guest_to_value(r0, &ret))
    }
}

fn field_key(name: &str, descriptor: &str) -> String {
    format!("{name}:{descriptor}")
}

/// Byte size of the ez-i `char[]` data block for `len` chars: `u32 len + u16[len]`.
pub(super) fn char_array_data_size(len: usize) -> u32 {
    4 + len as u32 * 2
}

/// Write the ez-i `char[]` data block at `data`: `[data] = len (u32)`, then each char
/// as a little-endian `u16` at `data + 4 + i*2` (the layout the glyph loop @0x10228
/// reads — cp30/cp31). Pure over guest memory.
pub(super) fn write_char_array_block(core: &mut ArmCore, data: u32, chars: &[u16]) -> Result<()> {
    write_generic(core, data, chars.len() as u32)?;
    for (i, &ch) in chars.iter().enumerate() {
        write_generic(core, data + 4 + i as u32 * 2, ch)?;
    }
    Ok(())
}
