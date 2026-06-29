//! `LgtJvmShared` — the process-wide LGT JVM glue state and its object-bridge
//! lifecycle (instance registry, native `new`/bind, getInstance singletons, lazy
//! init, value marshalling). The card render lifecycle lives in `render`; the
//! class model in `class_model`; the trampolines/tables in `dispatch`.

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::String,
    sync::Arc,
    vec::Vec,
};

use jvm::{ClassInstance, JavaType, JavaValue, Jvm};
use spin::Mutex;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, read_generic, write_generic};

use super::class_model::{LgtClassDefinition, LgtClassInstance, char_array_data_size, write_char_array_block};
use super::dispatch::TrampEntry;
use super::{AppFieldLayouts, FIELD_ARRAY_WORDS, OBJ_HEADER_SIZE, OBJ_PTR_FIELDS_OFFSET};
use crate::runtime::java::native_class::parse_native_class_from_handle;

/// Process-wide LGT JVM glue, shared (cheap `Arc` clones) between class
/// definitions, the trampoline SVC handler, and `java_load_classes`.
#[derive(Clone)]
pub struct LgtJvmShared {
    pub jvm: Jvm,
    #[allow(dead_code)] // kept for parity / future platform-service access
    pub system: System,
    /// guest object pointer -> its JVM instance.
    pub(super) instances: Arc<Mutex<BTreeMap<u32, Box<dyn ClassInstance>>>>,
    /// native -> platform method trampolines, indexed by SVC id.
    pub(super) trampolines: Arc<Mutex<Vec<TrampEntry>>>,
    /// Base of the global virtual-method offset table (`java_load_classes` output).
    /// Used as the vtable word for **app** objects, which extend the lcdui hierarchy
    /// and so dispatch through the union of all imported lcdui-hierarchy methods.
    pub(super) vmethod_table: Arc<Mutex<u32>>,
    /// Per-platform-class vtable base (only that class's own imported virtual methods
    /// at their global indices, everything else 0). A **platform proxy** object uses
    /// its class's vtable so an index that belongs to another class (e.g. Graphics'
    /// `drawLine`@14) does not misfire on, say, a `Runtime` — it reads 0 instead.
    pub(super) class_vtables: Arc<Mutex<BTreeMap<String, u32>>>,
    /// Guest object blocks allocated by the native `new` primitive (stdlib `0x32`)
    /// that have not yet been bound to a JVM instance. The constructor trampoline
    /// (`<init>`) binds them: it knows the class, so it instantiates and registers
    /// the JVM object for the pending guest pointer.
    pub(super) pending_new: Arc<Mutex<BTreeSet<u32>>>,
    /// Per app-class instance-field layout: `class -> [(field_name, field_type,
    /// object_slot)]`, where `object_slot = (app-ancestor field count) + declared
    /// index` (the flat guest-array slot the AOT lays the field out at, inherited
    /// fields first). Built in `register_app_classes`; `java_load_classes` consumes
    /// it to fill `field_offsets` for instance fields (de-aliasing the otherwise
    /// all-zero table — see `docs/lgt_abi.md` §5).
    pub(super) app_field_layouts: Arc<Mutex<AppFieldLayouts>>,
    /// Registered app-class inheritance graph: `class name -> parent name`. Built in
    /// `register_app_classes`; used by `resolve_card_class` to derive the shown card's
    /// app class from the class graph (the deepest app subclass of the platform `Card`)
    /// instead of assuming an obfuscated symbol name. See `docs/lgt_abi.md` §7.
    pub(super) app_class_graph: Arc<Mutex<BTreeMap<String, Option<String>>>>,
    /// `class descriptor handle -> singleton instance guest pointer`. The AOT's
    /// `getInstance` (java-interface import `0xc`) returns the one canonical instance
    /// of a class; it must be stable across calls (and threads) so per-class state
    /// (e.g. the `a.run` run-flag) is shared. Lazily created + cached here.
    pub(super) singletons: Arc<Mutex<BTreeMap<u32, u32>>>,
    /// Whether the shown card's initial scene has been entered yet (cp39). The per-frame
    /// driver runs the scene-enter `a(I)V` once on the first paint tick, then the
    /// per-frame step `aE()V` every tick. Set false until the first paint so the enter
    /// runs at a clean dispatch boundary (not mid-`a.run`, which clobbers the ARM core).
    pub(super) card_entered: Arc<Mutex<bool>>,
    /// Native code pointers of the shown card's lifecycle methods, resolved from the
    /// card class's descriptor at `show_card` time (NOT hardcoded): scene-enter `a(I)V`
    /// and per-frame step `aE()V`. `drive_card_step` reads these. `None` until a card
    /// is shown (or if the method isn't found on the card class). See `docs/lgt_abi.md`
    /// §7.
    pub(super) card_enter_ptr: Arc<Mutex<Option<u32>>>,
    pub(super) card_step_ptr: Arc<Mutex<Option<u32>>>,
    /// ELF `.data` segment range `(start, end)`, captured at load time. One of the two
    /// inputs to `register_app_classes` (the `.data` header scan); the other is the
    /// `0x07` registry. `None` until set by `set_data_range`. See `docs/lgt_abi.md` §3.
    pub(super) data_range: Arc<Mutex<Option<(u32, u32)>>>,
}

impl LgtJvmShared {
    pub fn new(jvm: Jvm, system: System) -> Self {
        Self {
            jvm,
            system,
            instances: Arc::new(Mutex::new(BTreeMap::new())),
            trampolines: Arc::new(Mutex::new(Vec::new())),
            vmethod_table: Arc::new(Mutex::new(0)),
            class_vtables: Arc::new(Mutex::new(BTreeMap::new())),
            pending_new: Arc::new(Mutex::new(BTreeSet::new())),
            app_field_layouts: Arc::new(Mutex::new(Vec::new())),
            app_class_graph: Arc::new(Mutex::new(BTreeMap::new())),
            singletons: Arc::new(Mutex::new(BTreeMap::new())),
            card_entered: Arc::new(Mutex::new(false)),
            card_enter_ptr: Arc::new(Mutex::new(None)),
            card_step_ptr: Arc::new(Mutex::new(None)),
            data_range: Arc::new(Mutex::new(None)),
        }
    }

    /// Record the ELF `.data` segment range, used as the scan input to
    /// `register_app_classes`. Called once at load time.
    pub fn set_data_range(&self, range: Option<(u32, u32)>) {
        *self.data_range.lock() = range;
    }

    /// java-interface import `0xd`: **lazy instance initialisation**. The AOT guards
    /// every lazy use with `if [inst.field+0x10] != 5 { 0xd(inst, init_fn) }` (`5` =
    /// "initialised"); run the instance initialiser `init_fn(inst)` once and mark it.
    /// Marks initialised *before* running so a re-entrant guard on the same instance
    /// doesn't recurse. See `docs/lgt_abi.md` §6.
    pub async fn lazy_instance_init(&self, core: &mut ArmCore, instance: u32, init_fn: u32) -> Result<()> {
        if instance == 0 || init_fn == 0 {
            return Ok(());
        }
        let fields = read_generic::<u32, _>(core, instance + OBJ_PTR_FIELDS_OFFSET)?;
        if fields == 0 {
            return Ok(());
        }
        let state = read_generic::<u16, _>(core, fields + 0x10)?;
        if state == 5 {
            return Ok(());
        }
        write_generic(core, fields + 0x10, 5u16)?;
        if let Err(e) = core.run_function::<u32>(init_fn, &[instance]).await {
            tracing::warn!("LGT lazy_instance_init inst={instance:#x} fn={init_fn:#x} failed: {e}");
        } else {
            tracing::debug!("LGT lazy_instance_init inst={instance:#x} fn={init_fn:#x} done");
        }
        Ok(())
    }

    /// java-interface import `0xb`: **lazy class initialisation**. The AOT guards class
    /// use with `if [[class+8]+0x1a] != 3 { 0xb(class) }` (`3` = "initialised"; the class
    /// header at `[class+8]` holds the state halfword at `+0x1a`). Mark the class
    /// initialised so the guard stops re-firing. See `docs/lgt_abi.md` §6.
    pub async fn lazy_class_init(&self, core: &mut ArmCore, class_handle: u32) -> Result<()> {
        if class_handle == 0 {
            return Ok(());
        }
        let header = read_generic::<u32, _>(core, class_handle + 8)?;
        if header == 0 {
            return Ok(());
        }
        let state = read_generic::<u16, _>(core, header + 0x1a)?;
        if state == 3 {
            return Ok(());
        }
        write_generic(core, header + 0x1a, 3u16)?;
        tracing::trace!("LGT lazy_class_init class={class_handle:#x} header={header:#x} (state {state}->3)");
        Ok(())
    }

    /// java-interface import `0xc` (`getInstance`): return the canonical singleton
    /// instance for the class identified by `class_handle` (`= class_header + 0x4c`).
    /// Created once (a bound app instance with its guest field array) and cached, so
    /// every `getInstance` call — across the main thread and spawned game-loop thread
    /// — sees the same object and its per-class state (run-flag etc.). Returns 0 if
    /// the class can't be resolved.
    pub async fn singleton_instance(&self, core: &mut ArmCore, class_handle: u32) -> u32 {
        if let Some(&p) = self.singletons.lock().get(&class_handle) {
            return p;
        }
        let name = match parse_native_class_from_handle(core, class_handle) {
            Ok(c) if !c.name.is_empty() => c.name,
            _ => return 0,
        };
        let class = match self.jvm.resolve_class(&name).await {
            Ok(c) => c,
            Err(_) => return 0,
        };
        let instance = match class.definition.instantiate(&self.jvm).await {
            Ok(i) => i,
            Err(_) => return 0,
        };
        let guest_ptr = instance.as_any().downcast_ref::<LgtClassInstance>().map(|o| o.guest_ptr).unwrap_or(0);
        if guest_ptr != 0 {
            self.singletons.lock().insert(class_handle, guest_ptr);
        }
        tracing::trace!("LGT getInstance({class_handle:#x}) = {name} @ {guest_ptr:#x} (created)");
        guest_ptr
    }

    pub(super) fn register_instance(&self, guest_ptr: u32, instance: Box<dyn ClassInstance>) {
        self.instances.lock().insert(guest_ptr, instance);
    }

    /// The native `new` primitive (stdlib `0x32` / java-interface `0xf`): allocate a
    /// guest object block (header + zeroed field array, global vtable word at `+0x00`)
    /// and mark it **pending** (unbound — no JVM class yet). The native code then calls
    /// the constructor, whose `<init>` trampoline runs [`Self::bind_pending`] to attach
    /// a JVM instance. This is the first half of the `new` → `<init>` bind lifecycle
    /// (see `docs/lgt_abi.md` §5).
    pub fn alloc_native_object(&self, core: &mut ArmCore) -> Result<u32> {
        let ptr_fields = Allocator::alloc(core, FIELD_ARRAY_WORDS * 4)?;
        wie_util::ByteWrite::write_bytes(core, ptr_fields, &[0u8; (FIELD_ARRAY_WORDS * 4) as usize])?;
        let ptr_raw = Allocator::alloc(core, OBJ_HEADER_SIZE)?;
        write_generic(core, ptr_raw, self.vtable_word())?;
        write_generic(core, ptr_raw + 4, 0u32)?;
        write_generic(core, ptr_raw + OBJ_PTR_FIELDS_OFFSET, ptr_fields)?;
        self.pending_new.lock().insert(ptr_raw);
        Ok(ptr_raw)
    }

    /// Second half of the `new` → `<init>` bind lifecycle: if `guest_ptr` is a pending
    /// native-`new` object, bind it to a fresh JVM instance of `class_name` (the
    /// constructor's class) and return it. App classes become an [`LgtClassInstance`]
    /// reusing this guest pointer (so native field writes and the JVM object share one
    /// block); platform classes are instantiated by the JVM and keyed by the guest
    /// pointer. If the class has a per-class override vtable, the object's `+0x00` word
    /// is repointed to it now (the class is finally known). See `docs/lgt_abi.md` §5.
    pub(super) async fn bind_pending(&self, core: &mut ArmCore, guest_ptr: u32, class_name: &str) -> Option<Box<dyn ClassInstance>> {
        if !self.pending_new.lock().remove(&guest_ptr) {
            return None;
        }
        // If the constructed platform class has a per-class vtable (hardcoded indices
        // that collide with the global table — e.g. StringBuffer's append@19), point
        // the guest object's `+0x00` at it now that the class is known. The object was
        // allocated by the native `new` primitive with the global vtable word.
        if let Some(&vt) = self.class_vtables.lock().get(class_name) {
            let _ = write_generic(core, guest_ptr, vt);
        }

        let class = self.jvm.resolve_class(class_name).await.ok()?;
        let definition = class.definition;
        let instance: Box<dyn ClassInstance> = if let Some(lgt) = definition.as_any().downcast_ref::<LgtClassDefinition>() {
            // app class: reuse the native guest block as the instance backing.
            Box::new(LgtClassInstance {
                guest_ptr,
                definition: lgt.clone(),
                jvm_fields: Arc::new(Mutex::new(BTreeMap::new())),
            })
        } else {
            // platform class: instantiate normally; the guest block is its handle.
            definition.instantiate(&self.jvm).await.ok()?
        };
        self.instances.lock().insert(guest_ptr, instance.clone());
        Some(instance)
    }
    /// Object `+0x00` value: the virtual-method table base (for AOT vtable dispatch).
    pub(super) fn vtable_word(&self) -> u32 {
        *self.vmethod_table.lock()
    }

    /// Map a JVM value to the guest word the native code expects (`this`/args).
    /// Object values become a guest pointer; a platform object with no guest
    /// backing yet gets a freshly-allocated proxy block.
    pub(super) fn value_to_guest(&self, core: &mut ArmCore, value: &JavaValue) -> u32 {
        match value {
            JavaValue::Void => 0,
            JavaValue::Boolean(x) => *x as u32,
            JavaValue::Byte(x) => *x as i32 as u32,
            JavaValue::Char(x) => *x as u32,
            JavaValue::Short(x) => *x as i32 as u32,
            JavaValue::Int(x) => *x as u32,
            JavaValue::Float(x) => x.to_bits(),
            JavaValue::Long(x) => *x as u32,
            JavaValue::Double(x) => x.to_bits() as u32,
            JavaValue::Object(None) => 0,
            JavaValue::Object(Some(inst)) => {
                if let Some(o) = inst.as_any().downcast_ref::<LgtClassInstance>() {
                    return o.guest_ptr;
                }
                self.register_platform_object(core, inst.clone())
            }
        }
    }

    /// Give a JVM platform object a guest identity: allocate an opaque proxy block
    /// whose vtable word is the object's own per-class vtable (so a hardcoded index
    /// that belongs to another class reads 0 rather than misfiring), register it in
    /// the instance map, and return the guest pointer. Returns 0 on allocation
    /// failure. Used both when marshalling a platform return/arg into native code and
    /// by the native String factory (java-interface import).
    pub fn register_platform_object(&self, core: &mut ArmCore, inst: Box<dyn ClassInstance>) -> u32 {
        let class_name = inst.class_definition().name();
        let vtable = self.class_vtables.lock().get(&class_name).copied().unwrap_or_else(|| self.vtable_word());
        match Allocator::alloc(core, OBJ_HEADER_SIZE) {
            Ok(ptr) => {
                let _ = write_generic(core, ptr, vtable);
                let _ = write_generic(core, ptr + 4, 0u32);
                let _ = write_generic(core, ptr + OBJ_PTR_FIELDS_OFFSET, 0u32);
                self.instances.lock().insert(ptr, inst);
                ptr
            }
            Err(_) => 0,
        }
    }

    /// Materialise a JVM `char[]` return into the ez-i guest layout the AOT reads for
    /// text (cp30/cp31 RE of the glyph loop @0x10228): an object whose `+0x08` points
    /// at a data block `{ u32 len, u16 chars[len] }` (chars start at data+4, stride 2,
    /// little-endian). Used for `String.toCharArray()[C` (slot 35) so the per-char draw
    /// loop reads real characters. The data block is allocated in the guest heap (same
    /// allocator as native `new`).
    ///
    /// PoC: the block is not freed (a per-frame text alloc can leak — TODO: pool/free
    /// once a frame boundary hook exists). Scoped to `char[]` only; other element types
    /// have different guest strides and must be RE-confirmed at their call site before
    /// being materialised (see `char_array_data_size` / `write_char_array_block`).
    pub(super) async fn materialize_char_array(&self, core: &mut ArmCore, array: Box<dyn ClassInstance>) -> Result<u32> {
        let jvm = self.jvm.clone();
        let len = match jvm.array_length(&array).await {
            Ok(l) => l,
            Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
        };
        let chars: Vec<u16> = match jvm.load_array(&array, 0, len).await {
            Ok(c) => c,
            Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
        };
        let data = Allocator::alloc(core, char_array_data_size(chars.len()))?;
        write_char_array_block(core, data, &chars)?;

        let obj = Allocator::alloc(core, OBJ_HEADER_SIZE)?;
        write_generic(core, obj, self.vtable_word())?;
        write_generic(core, obj + 4, 0u32)?;
        write_generic(core, obj + OBJ_PTR_FIELDS_OFFSET, data)?;
        self.instances.lock().insert(obj, array);
        Ok(obj)
    }

    /// Map a guest word back to a JVM value of the given type (args/returns from
    /// native into platform methods).
    pub(super) fn guest_to_value(&self, raw: u32, ty: &JavaType) -> JavaValue {
        match ty {
            JavaType::Void => JavaValue::Void,
            JavaType::Boolean => JavaValue::Boolean(raw != 0),
            JavaType::Byte => JavaValue::Byte(raw as i8),
            JavaType::Char => JavaValue::Char(raw as u16),
            JavaType::Short => JavaValue::Short(raw as i16),
            JavaType::Int => JavaValue::Int(raw as i32),
            JavaType::Float => JavaValue::Float(f32::from_bits(raw)),
            JavaType::Long => JavaValue::Long(raw as i64),
            JavaType::Double => JavaValue::Double(f64::from_bits(raw as u64)),
            JavaType::Class(_) | JavaType::Array(_) => {
                if raw == 0 {
                    JavaValue::Object(None)
                } else {
                    JavaValue::Object(self.instances.lock().get(&raw).cloned())
                }
            }
            _ => JavaValue::Void,
        }
    }
}
