//! LGT native-backed JVM (PoC): the AOT-compiled app's classes are registered as
//! JVM classes whose instances are guest(ARM)-memory-backed and whose methods
//! dispatch to ARM code; and the platform classes the app imports are exposed to
//! the native code through `java_load_classes`' fixed-offset function-pointer
//! tables (native -> platform trampolines).
//!
//! Object bridge:
//!  - App instance: a guest object block; `this+0x08` -> zeroed field array (the
//!    layout the AOT code expects: `r1=[this,#8]; str rX,[r1, idx<<2]`).
//!  - `instances` maps `guest_ptr -> ClassInstance` so a guest pointer flowing
//!    through native code (a `this`, an arg, a return) round-trips to its JVM
//!    object. Platform objects returned to native get a small proxy block.
//!
//! Dispatch directions:
//!  - JVM -> native: `LgtMethod::run` marshals args into `r0..r3`,
//!    `run_function(code_ptr)`, marshals the return.
//!  - native -> platform: `java_load_classes` writes a trampoline pointer into
//!    each requested method slot (`static/virtual_method_offsets[idx*4]`); calling
//!    it re-enters the JVM and invokes the matching `wie_wipi_java`/`wie_midp`
//!    method by name+descriptor.
//!
//! See `docs/lgt_abi.md` (consolidated ABI) and `docs/lgt_native_classes.md`
//! (descriptor byte layout).

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
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
use jvm::{ClassDefinition, ClassInstance, Field, JavaError, JavaType, JavaValue, Jvm, Method, Result as JvmResult, runtime::JavaLangString};
use spin::Mutex;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use super::native_class::{LgtNativeClass, parse_native_class};
use crate::runtime::SVC_CATEGORY_JAVA_TRAMPOLINE;

const OBJ_HEADER_SIZE: u32 = 0x0c;
const OBJ_PTR_FIELDS_OFFSET: u32 = 0x08;
const FIELD_ARRAY_WORDS: u32 = 256;

/// Per app-class instance-field layout: `(class_name, [(field_name, field_type,
/// object_slot)])`. See `LgtJvmShared::app_field_layouts`.
type AppFieldLayouts = Vec<(String, Vec<(String, String, u32)>)>;

// ---- shared runtime ----

/// Process-wide LGT JVM glue, shared (cheap `Arc` clones) between class
/// definitions, the trampoline SVC handler, and `java_load_classes`.
#[derive(Clone)]
pub struct LgtJvmShared {
    pub jvm: Jvm,
    #[allow(dead_code)] // kept for parity / future platform-service access
    pub system: System,
    /// guest object pointer -> its JVM instance.
    instances: Arc<Mutex<BTreeMap<u32, Box<dyn ClassInstance>>>>,
    /// native -> platform method trampolines, indexed by SVC id.
    trampolines: Arc<Mutex<Vec<TrampEntry>>>,
    /// Base of the global virtual-method offset table (`java_load_classes` output).
    /// Used as the vtable word for **app** objects, which extend the lcdui hierarchy
    /// and so dispatch through the union of all imported lcdui-hierarchy methods.
    vmethod_table: Arc<Mutex<u32>>,
    /// Per-platform-class vtable base (only that class's own imported virtual methods
    /// at their global indices, everything else 0). A **platform proxy** object uses
    /// its class's vtable so an index that belongs to another class (e.g. Graphics'
    /// `drawLine`@14) does not misfire on, say, a `Runtime` — it reads 0 instead.
    class_vtables: Arc<Mutex<BTreeMap<String, u32>>>,
    /// Guest object blocks allocated by the native `new` primitive (stdlib `0x32`)
    /// that have not yet been bound to a JVM instance. The constructor trampoline
    /// (`<init>`) binds them: it knows the class, so it instantiates and registers
    /// the JVM object for the pending guest pointer.
    pending_new: Arc<Mutex<BTreeSet<u32>>>,
    /// Per app-class instance-field layout: `class -> [(field_name, field_type,
    /// object_slot)]`, where `object_slot = (app-ancestor field count) + declared
    /// index` (the flat guest-array slot the AOT lays the field out at, inherited
    /// fields first). Built in `register_app_classes`; `java_load_classes` consumes
    /// it to fill `field_offsets` for instance fields (de-aliasing the otherwise
    /// all-zero table — see `docs/lgt_abi.md` §5).
    app_field_layouts: Arc<Mutex<AppFieldLayouts>>,
    /// `class descriptor handle -> singleton instance guest pointer`. The AOT's
    /// `getInstance` (java-interface import `0xc`) returns the one canonical instance
    /// of a class; it must be stable across calls (and threads) so per-class state
    /// (e.g. the `a.run` run-flag) is shared. Lazily created + cached here.
    singletons: Arc<Mutex<BTreeMap<u32, u32>>>,
    /// Whether the shown card's initial scene has been entered yet (cp39). The per-frame
    /// driver runs the scene-enter `i.a(I)V` once on the first paint tick, then the
    /// per-frame step `i.aE()V` every tick. Set false until the first paint so the enter
    /// runs at a clean dispatch boundary (not mid-`a.run`, which clobbers the ARM core).
    card_entered: Arc<Mutex<bool>>,
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
            singletons: Arc::new(Mutex::new(BTreeMap::new())),
            card_entered: Arc::new(Mutex::new(false)),
        }
    }

    /// java-interface import `0xd` (cp51): **lazy instance initialisation**. The AOT
    /// guards every lazy use of an instance with `if [inst.field+0x10] != 5 { 0xd(inst,
    /// init_fn) }` — `5` = "initialised". `0xd` runs the instance's initialiser
    /// `init_fn(inst)` once, then marks it initialised. Left as a no-op (cp50), every
    /// getInstance singleton stayed uninitialised, so its fields (e.g. the scene-object
    /// count `field[0x44]`) were 0 ⇒ empty scene ⇒ `[+0xd4]` never populated ⇒ no
    /// sprites/text. `init_fn` is passed by the call site (e.g. `i.c` @0x788a0 for the
    /// `b`/`o` card singleton). Marks initialised *before* running so a re-entrant
    /// lazy-init guard on the same instance doesn't recurse.
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

    /// java-interface import `0xb` (cp51): **lazy class initialisation**. The AOT guards
    /// class use with `if [[class+8]+0x1a] != 3 { 0xb(class) }` (`3` = "initialised";
    /// the class header at `[class+8]` holds the state halfword at `+0x1a`). No-op'd, the
    /// flag never reached 3, so the guard re-fired on every access (3665× per run — a
    /// spin) and the class never set up. Mark the class initialised so the guard passes.
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
        let name = match super::native_class::parse_native_class_from_handle(core, class_handle) {
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

    fn register_instance(&self, guest_ptr: u32, instance: Box<dyn ClassInstance>) {
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
    async fn bind_pending(&self, core: &mut ArmCore, guest_ptr: u32, class_name: &str) -> Option<Box<dyn ClassInstance>> {
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

    /// LGT java-interface import `0x57` (show-card): the app `new`'d a card and hands
    /// it to the platform to display (observed `0x57(jlet, card_guest, jlet)` in
    /// `a.run` — cp39 trace). The guest block was bound to the platform `Card` base by
    /// its `<init>` trampoline: only the `super Card.<init>` runs through wie, so the
    /// app's most-derived class isn't visible at bind time and `paint` would resolve to
    /// the empty platform `Card.paint`. Rebind the guest block to the app's title-card
    /// class (`i` — cp38) as an [`LgtClassInstance`] reusing the SAME guest pointer, so
    /// `paint` dispatches through `i -> b -> o` to the native `o.paint` (the real draw
    /// @0xd8d70), then push it to wie's `Display` so the MIDP paint loop ticks it each
    /// frame. Left no-op before cp39, which is why `o.paint` never ran. See
    /// `docs/lgt_abi.md` §7.
    pub async fn show_card(&self, card_guest: u32) -> Result<()> {
        if card_guest == 0 {
            return Ok(());
        }
        // `0x57` is overloaded: the Game-flow call passes a real card guest object
        // (heap-allocated, already bound by its `<init>`), but `a.run` also calls
        // `0x57(jlet, <carried code ptr>, jlet)` where arg1 is a `.text` code pointer,
        // not a card. Only act on a genuine bound object (present in `instances`) so the
        // carried-code call is ignored.
        if !self.instances.lock().contains_key(&card_guest) {
            tracing::trace!("LGT show_card: {card_guest:#x} is not a bound object (carried-code 0x57?); ignoring");
            return Ok(());
        }
        // App card class to rebind to. For the current reach (title) this is the title
        // card `i` (cp38); other cards would each need their class resolved once shown.
        const CARD_CLASS: &str = "i";
        let jvm = self.jvm.clone();

        let class = match jvm.resolve_class(CARD_CLASS).await {
            Ok(c) => c,
            Err(_) => {
                tracing::warn!("LGT show_card: app card class {CARD_CLASS:?} not registered; card {card_guest:#x} left as platform Card");
                return Ok(());
            }
        };
        let definition = match class.definition.as_any().downcast_ref::<LgtClassDefinition>() {
            Some(def) => def.clone(),
            None => {
                tracing::warn!("LGT show_card: {CARD_CLASS:?} is not an app class; card {card_guest:#x} left as-is");
                return Ok(());
            }
        };
        let card: Box<dyn ClassInstance> = Box::new(LgtClassInstance {
            guest_ptr: card_guest,
            definition,
            jvm_fields: Arc::new(Mutex::new(BTreeMap::new())),
        });
        self.instances.lock().insert(card_guest, card.clone());
        tracing::debug!("LGT show_card: rebound card {card_guest:#x} -> {CARD_CLASS:?}; pushing to Display");

        let display_val: JavaValue = match jvm
            .invoke_static(
                "org/kwis/msp/lcdui/Display",
                "getDefaultDisplay",
                "()Lorg/kwis/msp/lcdui/Display;",
                Vec::<JavaValue>::new(),
            )
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
        };
        let display = match display_val {
            JavaValue::Object(Some(d)) => d,
            _ => {
                tracing::warn!("LGT show_card: getDefaultDisplay returned null");
                return Ok(());
            }
        };
        let args = alloc::vec![JavaValue::Object(Some(card))];
        let _: JavaValue = match jvm.invoke_virtual(&display, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", args).await {
            Ok(v) => v,
            Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
        };
        // The scene-enter is driven from the first paint tick (a clean dispatch
        // boundary), not here: `show_card` runs mid-`a.run` and driving native code on
        // the same core would clobber that in-flight context. See `drive_card_step`.
        *self.card_entered.lock() = false;
        Ok(())
    }

    /// Per-frame card step (cp39): run the shown card's step method `i.aE()V`
    /// (@0x72f2c) on the card `this`. ez-i ticks the current card each frame — advance
    /// the scene state machine, then paint — but wie only drives `paint`, so we run the
    /// step here, immediately before the card's `o.paint` (see [`LgtMethod::run`]).
    /// `i.aE` is the per-frame state machine that reaches the `o.g` render-gate setter
    /// (one of the four `0xdb200` callers — cp38); `o.paint` early-returns forever while
    /// `o.g==0`, so without this tick the card never renders. Errors are logged and
    /// swallowed so a step fault doesn't abort the frame. See `docs/lgt_abi.md` §7.
    pub async fn drive_card_step(&self, core: &mut ArmCore, card_this: u32) {
        if card_this == 0 {
            return;
        }
        const SCENE_ENTER_PTR: u32 = 0x1d4ac; // i.a(I)V — scene-enter (cp38)
        const INITIAL_SCENE: u32 = 0; // initial/title scene (state arg, used deeper)
        const STEP_PTR: u32 = 0x72f2c; // i.aE()V — per-frame step (cp38)

        // First tick: enter the initial scene. `i.a(I)V`'s prologue sets the `o.g`
        // render gate (via the @0xdb200 setter) and runs the scene setup that
        // initialises the state the per-frame step advances; without it `i.aE`
        // early-returns on uninitialised state and `o.paint` stays gated (cp38).
        let need_enter = {
            let mut entered = self.card_entered.lock();
            if *entered {
                false
            } else {
                *entered = true;
                true
            }
        };
        if need_enter {
            let r: Result<u32> = core.run_function(SCENE_ENTER_PTR, &[card_this, INITIAL_SCENE]).await;
            match r {
                Ok(_) => tracing::debug!("LGT drive_card_step: entered scene via i.a({card_this:#x}, {INITIAL_SCENE})"),
                Err(e) => tracing::warn!("LGT scene-enter i.a @{SCENE_ENTER_PTR:#x} this={card_this:#x} failed: {e}"),
            }
        }

        let r: Result<u32> = core.run_function(STEP_PTR, &[card_this]).await;
        if let Err(e) = r {
            tracing::warn!("LGT card step i.aE @{STEP_PTR:#x} this={card_this:#x} failed: {e}");
        }

        // cp44: sustain the frame loop. `a.run` is one-shot and the app's per-frame
        // repaint request (import `0xe2`) is not yet wired, so wie paints a few times
        // then idles. Schedule the next repaint so the card's step+paint run each frame
        // — a legitimate continuous render mirroring the platform's per-frame tick (not
        // a force; `repaint` only enqueues a paint event, which runs after this one).
        let card = self.instances.lock().get(&card_this).cloned();
        if let Some(card) = card {
            let _: core::result::Result<JavaValue, JavaError> = self.jvm.invoke_virtual(&card, "repaint", "()V", Vec::<JavaValue>::new()).await;
        }
    }

    /// Object `+0x00` value: the virtual-method table base (for AOT vtable dispatch).
    fn vtable_word(&self) -> u32 {
        *self.vmethod_table.lock()
    }

    /// Map a JVM value to the guest word the native code expects (`this`/args).
    /// Object values become a guest pointer; a platform object with no guest
    /// backing yet gets a freshly-allocated proxy block.
    fn value_to_guest(&self, core: &mut ArmCore, value: &JavaValue) -> u32 {
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
    async fn materialize_char_array(&self, core: &mut ArmCore, array: Box<dyn ClassInstance>) -> Result<u32> {
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
    fn guest_to_value(&self, raw: u32, ty: &JavaType) -> JavaValue {
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

// ---- class metadata (pure Rust) ----

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
    fn from_native(class: &LgtNativeClass, core: ArmCore, shared: LgtJvmShared) -> Self {
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
    guest_ptr: u32,
    definition: LgtClassDefinition,
    // JVM-side field storage. Not yet unified with the guest field array at
    // `guest_ptr` (see `docs/lgt_abi.md` §5): fields written by ARM code and via the
    // JVM currently live in separate stores. For the current reach (boot + setup) the
    // two paths don't alias the same field, so this hasn't surfaced.
    jvm_fields: Arc<Mutex<BTreeMap<String, JavaValue>>>,
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
fn char_array_data_size(len: usize) -> u32 {
    4 + len as u32 * 2
}

/// Write the ez-i `char[]` data block at `data`: `[data] = len (u32)`, then each char
/// as a little-endian `u16` at `data + 4 + i*2` (the layout the glyph loop @0x10228
/// reads — cp30/cp31). Pure over guest memory.
fn write_char_array_block(core: &mut ArmCore, data: u32, chars: &[u16]) -> Result<()> {
    write_generic(core, data, chars.len() as u32)?;
    for (i, &ch) in chars.iter().enumerate() {
        write_generic(core, data + 4 + i as u32 * 2, ch)?;
    }
    Ok(())
}

// ---- native -> platform trampolines ----

#[derive(Clone)]
struct TrampEntry {
    class_name: String,
    name: String,
    descriptor: String,
    is_virtual: bool,
}

/// SVC handler for a native -> platform call. The SVC id selects the trampoline;
/// `r0..` carry `this`(virtual/<init>) + args; the result goes back in `r0`.
pub async fn handle_java_trampoline(core: &mut ArmCore, shared: &mut LgtJvmShared, id: SvcId) -> Result<u32> {
    let (_, lr) = core.read_pc_lr()?;
    let entry = {
        let table = shared.trampolines.lock();
        match table.get(id.0 as usize) {
            Some(e) => e.clone(),
            None => return Err(WieError::FatalError(format!("LGT trampoline: unknown id {}", id.0))),
        }
    };

    // A null/placeholder slot the app declared but does not implement: no-op.
    if entry.name.is_empty() {
        tracing::trace!("LGT trampoline noop slot {}", id.0);
        core.set_next_pc(lr)?;
        return Ok(0);
    }

    let arg_types = match JavaType::parse(&entry.descriptor) {
        JavaType::Method(a, _) => a,
        _ => Vec::new(),
    };

    // `this` first for virtual/<init>; then one guest word per arg slot.
    let is_static = !entry.is_virtual && entry.name != "<init>";
    let mut pos = 0usize;
    let mut this_raw = 0u32;
    let mut this = if is_static {
        None
    } else {
        this_raw = core.read_param(pos)?;
        pos += 1;
        shared.instances.lock().get(&this_raw).cloned()
    };

    // `obj = new50(); obj.<init>()`: the native object allocator hands `<init>` a
    // pending guest block. Bind it to a JVM instance of the constructed class now.
    if this.is_none() && entry.name == "<init>" && this_raw != 0 {
        this = shared.bind_pending(core, this_raw, &entry.class_name).await;
    }
    if this.is_none() && !is_static && entry.name != "<init>" && this_raw != 0 {
        let pending = shared.pending_new.lock().contains(&this_raw);
        // A native-`new`'d object (`pending_new`) that was initialised by raw native
        // code (no platform `<init>` trampoline) and is now the target of a DIRECT
        // hardcoded `vtable[N]` call. Its class is compiled away (cp12/13), so the
        // global by-name vtable misroutes slot N to a foreign platform method (here
        // Graphics.getClip*). RE proved (cp14) the result is DISCARDED at this call
        // pattern (`bx ip` then `ldr r0,[fp,#-0x2c]` overwrites it) — i.e. a no-op
        // probe. Pass it through (return 0) instead of NPE-ing, per the autopilot's
        // r8 pass-through rule. Scoped to `pending_new` so genuinely-bound platform
        // calls (which need a real value, e.g. layout getHeight) never reach here.
        if pending {
            tracing::debug!(
                "LGT pending-new probe {}.{} this={this_raw:#x} lr={lr:#x} -> 0 (discarded, cp14)",
                entry.class_name,
                entry.name
            );
            core.set_next_pc(lr)?;
            return Ok(0);
        }
        let vt = read_generic::<u32, _>(core, this_raw).unwrap_or(0);
        let global = *shared.vmethod_table.lock();
        tracing::warn!(
            "LGT UNBOUND this for {}.{}{}: this_raw={this_raw:#x} vtable_word={vt:#x} (global={global:#x}) lr={lr:#x}",
            entry.class_name,
            entry.name,
            entry.descriptor
        );
    }
    let mut jargs = Vec::with_capacity(arg_types.len());
    for ty in &arg_types {
        let raw = core.read_param(pos)?;
        pos += 1;
        if matches!(ty, JavaType::Long | JavaType::Double) {
            pos += 1; // 64-bit args take two slots (low word used)
        }
        jargs.push(shared.guest_to_value(raw, ty));
    }

    let this_class = this.as_ref().map(|t| t.class_definition().name());
    tracing::debug!(
        "LGT trampoline id={} -> {}.{}{}  this_raw={this_raw:#x} this_actual={:?} lr={lr:#x}",
        id.0,
        entry.class_name,
        entry.name,
        entry.descriptor,
        this_class
    );

    let jvm = shared.jvm.clone();

    // wie's StringBuffer implements `append([CII)` but not `append(Ljava/lang/String;)`
    // / `append(Object)`, which the AOT calls (vtable[19]). Synthesise it from the
    // argument String's chars so the builder chain (`"txt/"+arg+".dat"`) works.
    if entry.class_name == "java/lang/StringBuffer" && entry.name == "append" && entry.descriptor.starts_with("(Ljava/lang/") {
        let result = stringbuffer_append_string(&jvm, this, jargs.into_iter().next()).await;
        let result = match result {
            Ok(v) => v,
            Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
        };
        let r0 = shared.value_to_guest(core, &result);
        core.set_next_pc(lr)?;
        return Ok(r0);
    }

    let result: core::result::Result<JavaValue, JavaError> = if entry.name == "<init>" {
        match &this {
            Some(this) => jvm.invoke_special(this, &entry.class_name, "<init>", &entry.descriptor, jargs).await,
            None => Err(jvm.exception("java/lang/NullPointerException", "<init> without this").await),
        }
    } else if is_static {
        jvm.invoke_static(&entry.class_name, &entry.name, &entry.descriptor, jargs).await
    } else {
        match &this {
            Some(this) => jvm.invoke_virtual(this, &entry.name, &entry.descriptor, jargs).await,
            None => Err(jvm.exception("java/lang/NullPointerException", &entry.name).await),
        }
    };

    // No no-op fallback (it diverges — confirmed: a 0 return for getHeight loops the
    // layout). Unresolved calls fail loudly so the blocker stays visible.
    let result = match result {
        Ok(v) => v,
        Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
    };

    // A `char[]` return (e.g. `String.toCharArray()`) must be materialised into the
    // ez-i guest layout `{u32 len, u16 chars}` the AOT reads, not the empty proxy a
    // plain object marshals to — otherwise the text glyph loop reads zero chars (cp31).
    let r0 = match &result {
        JavaValue::Object(Some(o)) if o.class_definition().name() == "[C" => shared.materialize_char_array(core, o.clone()).await?,
        _ => shared.value_to_guest(core, &result),
    };
    core.set_next_pc(lr)?;
    Ok(r0)
}

/// Synthesise `StringBuffer.append(String)` (absent in wie's StringBuffer) by routing
/// through the existing `append([CII)`: read the argument String's chars and append
/// them. `append(null)` appends the text "null" (Java semantics). Returns the
/// StringBuffer (the chainable result the AOT expects).
async fn stringbuffer_append_string(
    jvm: &Jvm,
    this: Option<Box<dyn ClassInstance>>,
    arg: Option<JavaValue>,
) -> core::result::Result<JavaValue, JavaError> {
    let this = match this {
        Some(t) => t,
        None => return Err(jvm.exception("java/lang/NullPointerException", "StringBuffer.append this").await),
    };
    let str_obj: Box<dyn ClassInstance> = match arg {
        Some(JavaValue::Object(Some(s))) => s,
        _ => JavaLangString::from_rust_string(jvm, "null").await?,
    };
    let chars = jvm.invoke_virtual(&str_obj, "toCharArray", "()[C", Vec::<JavaValue>::new()).await?;
    let chars_arr = match chars {
        JavaValue::Object(Some(a)) => a,
        _ => return Err(jvm.exception("java/lang/NullPointerException", "toCharArray result").await),
    };
    let len = jvm.array_length(&chars_arr).await? as i32;
    let args = alloc::vec![JavaValue::Object(Some(chars_arr)), JavaValue::Int(0), JavaValue::Int(len)];
    jvm.invoke_virtual(&this, "append", "([CII)Ljava/lang/StringBuffer;", args).await
}

// ---- registration + table install ----

pub fn register_java_trampoline_handler(core: &mut ArmCore, shared: &LgtJvmShared) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA_TRAMPOLINE, handle_java_trampoline, shared)
}

/// Empirically-identified vtable slots for `java/lang/*` classes whose layout is
/// NOT in the app's import data (they declare 0 imported virtual methods) but which
/// the AOT calls by hardcoded vtable index. Each entry is `(vtable_index, name,
/// descriptor)`. These are **estimates (추정)** grounded in how the native code uses
/// the call (see `docs/lgt_abi.md` §4), not a derived spec — extend as more
/// (class, index) pairs are observed.
fn known_java_lang_vtable(class: &str) -> &'static [(u32, &'static str, &'static str)] {
    match class {
        // Game.<init> startup: getRuntime().<14>() result discarded (void => gc),
        // then getRuntime().<13>() result used as a value (=> freeMemory).
        "java/lang/Runtime" => &[(13, "freeMemory", "()J"), (14, "gc", "()V")],
        // cp10: `new StringBuffer(); sb.append(s1).append(s2).append(s3).toString()`
        // at 0x4720 builds `"txt/" + arg + ".dat"`. Disassembly: vtable[19] (offset
        // 0x4c) is called with one String arg and returns the StringBuffer (chained
        // 3x) => `append(String)`; vtable[5] (offset 0x14) on the result is then read
        // as a String => `toString()`. Behaviour-confirmed via the constant pool the
        // append args come from. `append(String)` is synthesised in the trampoline
        // (wie's StringBuffer has only `append([CII)`), see `handle_java_trampoline`.
        // cp40: `i.a(Z)V` (@0x2fd94) builds resource paths
        // (`append(String).append(int)…` → e.g. `"txt/" + id + ".dat"`): physical
        // slot 19 (offset 0x4c) = `append(String)`, physical slot 24 (offset 0x60) =
        // `append(int)`. RE'd by the call shape (`r5.slot24(r1)` returns the chainable
        // receiver, `r1 = singleton.field[0x74]`) and a runtime probe (raw arg = `8`, a
        // small int). Without slot 24 the call fell through to global slot 24 =
        // `Display.pushCard` → fatal, blocking the scene's data/sprite loads.
        "java/lang/StringBuffer" => &[
            (5, "toString", "()Ljava/lang/String;"),
            (19, "append", "(Ljava/lang/String;)Ljava/lang/StringBuffer;"),
            (24, "append", "(I)Ljava/lang/StringBuffer;"),
        ],
        // cp18: a.startApp tail does `t = new Thread(this); t.<11>()` then returns
        // (the call's result is discarded; the Runnable is the Jlet base `a`, whose
        // `run()` is the game loop). vtable[11] (offset 0x2c) on a freshly-constructed
        // Thread, result unused => `start()V`. Behaviour-confirmed (Thread spawns the
        // Runnable). Thread declares 0 imported virtuals, so it needs this per-class slot.
        "java/lang/Thread" => &[(11, "start", "()V")],
        // cp30: the draw-text wrapper `B(Graphics, String)` (@0x100d8) calls
        // `s.vtable[physical 35]()` (no args) then iterates the result as a char array:
        // `r2 = [ret+8]; len = [r2]; for i in 0..len { char = [r2 + i*2 + 4] }` — a
        // per-char glyph loop (bitmap font). The no-arg, char-array-returning String
        // method is `toCharArray()[C`. Behaviour-RE'd cp30 (the loop reads `[ret+8]` as
        // `{u32 len, u16 chars[]}` and bounds on `len`). String declares 0 imported
        // virtuals, so it needs this per-class slot.
        "java/lang/String" => &[(35, "toCharArray", "()[C")],
        _ => &[],
    }
}

fn read_pair(core: &ArmCore, base: u32, idx: u32) -> (Option<String>, Option<String>) {
    let n = read_generic::<u32, _>(core, base + idx * 8).unwrap_or(0);
    let t = read_generic::<u32, _>(core, base + idx * 8 + 4).unwrap_or(0);
    (read_cstr(core, n), read_cstr(core, t))
}

fn read_cstr(core: &ArmCore, ptr: u32) -> Option<String> {
    if ptr == 0 {
        return None;
    }
    let bytes = read_null_terminated_string_bytes(core, ptr).ok()?;
    if bytes.is_empty() || !bytes.iter().all(|&b| (0x20..0x7f).contains(&b)) {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// Number of virtual-method references to build (the `virtual_method_offsets` table
/// is ~102 halfwords; this covers it with margin).
const VTABLE_REFS: u32 = 128;

/// Physical vtable word index for virtual method-ref `r`. The AOT dispatches via
/// `vtable[virtual_method_offsets[r] + 1]` (a literal `ldr ip,[r3,#4]` after
/// `add r3, r3, idx<<2`), so physical slot 0 is RESERVED and refs start at slot 1.
/// With `virtual_method_offsets[r] = r`, dispatch reads `vtable[r + 1]` — the slot
/// this returns. Centralising the `+1` keeps install and dispatch in lock-step (a
/// prior off-by-one here misrouted ref 6 to ref 7). See `docs/lgt_abi.md` §4.
const fn physical_vtable_slot(r: u32) -> u32 {
    r + 1
}

/// Implement `java_load_classes` with the two-level virtual-dispatch model
/// (see `docs/lgt_abi.md` §4):
///
/// - **Virtual** (`r3=[this]; bx [r3 + idx*4]`, optionally `idx =
///   virtual_method_offsets[ref]`): the object's `+0x00` points to a **pointer
///   vtable** indexed by the global `virtual_methods` array position; each slot is a
///   trampoline that `invoke_virtual`s that method *by name* on `this`. So one
///   global vtable serves every object (platform proxy → wie method; app object →
///   native ARM method), and `virtual_method_offsets[ref] = ref` (identity).
/// - **Static** (`bx [static_method_offsets + i*4]`): direct function pointers.
/// - **java/lang** classes the AOT calls by a hardcoded index that collides with
///   another class's slot (Runtime 13/14) get a per-class vtable (copy of the global
///   one with the [`known_java_lang_vtable`] slots overridden).
#[allow(clippy::too_many_arguments)]
pub fn install_platform_tables(
    core: &mut ArmCore,
    shared: &LgtJvmShared,
    classes: u32,
    fields: u32,
    virtual_methods: u32,
    static_methods: u32,
    field_offsets: u32,
    virtual_method_offsets: u32,
    static_method_offsets: u32,
) -> Result<()> {
    let count = read_generic::<u32, _>(core, classes).unwrap_or(0);
    tracing::debug!("install_platform_tables: {count} imported classes");

    // Gather the imported-class method/field ranges.
    struct Cls {
        name: String,
        vmo: u32,
        sfo: u32,
        sfc: u32,
        smo: u32,
        smc: u32,
        vmc: u32,
    }
    let mut classes_vec = Vec::new();
    for i in 0..count {
        let base = classes + 4 + i * 24;
        if let Some(name) = read_cstr(core, read_generic::<u32, _>(core, base).unwrap_or(0)) {
            classes_vec.push(Cls {
                name,
                sfo: read_generic::<u16, _>(core, base + 8).unwrap_or(0) as u32,
                sfc: read_generic::<u16, _>(core, base + 10).unwrap_or(0) as u32,
                vmo: read_generic::<u16, _>(core, base + 12).unwrap_or(0) as u32,
                vmc: read_generic::<u16, _>(core, base + 14).unwrap_or(0) as u32,
                smo: read_generic::<u16, _>(core, base + 20).unwrap_or(0) as u32,
                smc: read_generic::<u16, _>(core, base + 22).unwrap_or(0) as u32,
            });
        }
    }
    // ref -> declaring platform class (for logging only; dispatch is by name).
    let vref_class = |r: u32| -> String {
        classes_vec
            .iter()
            .find(|c| c.vmo <= r && r < c.vmo + c.vmc)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "app".into())
    };

    let mut method_slots = 0usize;
    let mut field_slots = 0usize;

    // 1) Global virtual vtable + identity index table.
    //
    // RESERVED SLOT 0 (cp15): the AOT's offset-table virtual dispatch is
    // `idx = virtual_method_offsets[ref]; bx vtable[idx + 1]` — the `+1` (a literal
    // `ldr ip,[r3,#4]` after `add r3, r3, idx<<2`) means physical vtable slot 0 is
    // reserved and methods start at slot 1. So method-ref `r` lives at PHYSICAL slot
    // `r + 1`, with `virtual_method_offsets[r] = r` (the logical index). Then
    // `vtable[offset[r] + 1] = vtable[r + 1]` correctly invokes ref `r` (verified by
    // RE of AnnunciatorComponent.show@ref6 → was misrouting to vtable[7]=File.read).
    let global_vtable = Allocator::alloc(core, (VTABLE_REFS + 1) * 4)?;
    wie_util::ByteWrite::write_bytes(core, global_vtable, &[0u8; ((VTABLE_REFS + 1) * 4) as usize])?;
    for r in 0..VTABLE_REFS {
        let (mname, mtype) = read_pair(core, virtual_methods, r);
        if let (Some(mname), Some(mtype)) = (mname, mtype)
            && mtype.starts_with('(')
        {
            let cls = vref_class(r);
            let stub = make_method_trampoline(core, shared, &cls, Some(mname), Some(mtype), true)?;
            write_generic(core, global_vtable + physical_vtable_slot(r) * 4, stub)?; // reserved slot 0
            // Logical index of method-ref `r` is `r`; the dispatch adds the reserved
            // slot (`+1`). Only written for real method refs to stay in the table.
            write_generic(core, virtual_method_offsets + r * 2, r as u16)?;
            method_slots += 1;
        }
    }
    // Every object's `+0x00` points here (app objects + platform proxies).
    *shared.vmethod_table.lock() = global_vtable;

    // 2) Static methods (direct pointers) + static-field slots, per imported class.
    for c in &classes_vec {
        for j in 0..c.smc {
            let idx = c.smo + j;
            let (mname, mtype) = read_pair(core, static_methods, idx);
            let stub = make_method_trampoline(core, shared, &c.name, mname, mtype, false)?;
            write_generic(core, static_method_offsets + idx * 4, stub)?;
            method_slots += 1;
        }
        // Static-field slots only (identity fill). A blanket identity fill of the
        // *instance* field table regressed a.startApp (instance field semantics are
        // more subtle — handled separately by the inheritance-aware pass in step 5).
        for j in 0..c.sfc {
            let idx = c.sfo + j;
            write_generic(core, field_offsets + idx * 2, (idx % FIELD_ARRAY_WORDS) as u16)?;
            field_slots += 1;
        }
    }

    // 4) java/lang per-class override vtables: copy the global vtable, then override
    //    the empirically-identified hardcoded slots (추정; see `known_java_lang_vtable`
    //    and `docs/lgt_abi.md` §4).
    for c in &classes_vec {
        let known = known_java_lang_vtable(&c.name);
        if known.is_empty() {
            continue;
        }
        let vt = Allocator::alloc(core, (VTABLE_REFS + 1) * 4)?;
        let mut buf = alloc::vec![0u8; ((VTABLE_REFS + 1) * 4) as usize];
        wie_util::ByteRead::read_bytes(core, global_vtable, &mut buf)?;
        wie_util::ByteWrite::write_bytes(core, vt, &buf)?;
        // `known` indices are PHYSICAL slots (the AOT direct-dispatches `vtable[idx]`
        // with the reserved slot already baked into `idx`, e.g. StringBuffer.append at
        // physical 19, Runtime.freeMemory at physical 13).
        for &(idx, mname, mtype) in known {
            let stub = make_method_trampoline(core, shared, &c.name, Some(mname.into()), Some(mtype.into()), true)?;
            if idx <= VTABLE_REFS {
                write_generic(core, vt + idx * 4, stub)?;
            }
        }
        shared.class_vtables.lock().insert(c.name.clone(), vt);
    }

    // 5) Instance field_offsets. The AOT addresses instance fields as
    //    `obj.field[field_offsets[K]]`; left all-zero, every field-ref aliases slot 0
    //    (e.g. the `a.startApp` "is Display set?" gate read a sibling field and wrongly
    //    skipped setup — cp16/cp17). The `fields` array is grouped by owning app class;
    //    segment it by matching each window to a class's field set, then write the
    //    inherited-first object slot computed in `register_app_classes`.
    let layouts = shared.app_field_layouts.lock().clone();
    if !layouts.is_empty() {
        // Read the flat (name, type) field-ref array until a null entry.
        let mut refs: Vec<(String, String)> = Vec::new();
        for k in 0..1024u32 {
            let n = read_cstr(core, read_generic::<u32, _>(core, fields + k * 8).unwrap_or(0));
            let t = read_cstr(core, read_generic::<u32, _>(core, fields + k * 8 + 4).unwrap_or(0));
            match (n, t) {
                (Some(n), Some(t)) => refs.push((n, t)),
                _ => break,
            }
        }
        let mut i = 0usize;
        let mut filled = 0usize;
        'outer: while i < refs.len() {
            // Find an app class whose field set exactly matches the window at `i`.
            for (_, fl) in &layouts {
                let n = fl.len();
                if n == 0 || i + n > refs.len() {
                    continue;
                }
                let window: BTreeSet<(&str, &str)> = refs[i..i + n].iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
                let set: BTreeSet<(&str, &str)> = fl.iter().map(|(a, b, _)| (a.as_str(), b.as_str())).collect();
                if window.len() == n && window == set {
                    let slot_of: BTreeMap<(&str, &str), u32> = fl.iter().map(|(a, b, s)| ((a.as_str(), b.as_str()), *s)).collect();
                    for (j, (rn, rt)) in refs[i..i + n].iter().enumerate() {
                        if let Some(&slot) = slot_of.get(&(rn.as_str(), rt.as_str())) {
                            write_generic(core, field_offsets + (i + j) as u32 * 2, slot as u16)?;
                            filled += 1;
                        }
                    }
                    i += n;
                    continue 'outer;
                }
            }
            i += 1; // unmatched ref (rare tail entry) — leave as-is
        }
        tracing::debug!("LGT java_load_classes: filled {filled}/{} instance field_offsets", refs.len());
    }

    tracing::info!("LGT java_load_classes: filled {method_slots} method slots, {field_slots} field slots (two-level vtable)");
    Ok(())
}

fn make_method_trampoline(
    core: &mut ArmCore,
    shared: &LgtJvmShared,
    class_name: &str,
    mname: Option<String>,
    mtype: Option<String>,
    is_virtual: bool,
) -> Result<u32> {
    let entry = match (mname, mtype) {
        (Some(name), Some(descriptor)) => TrampEntry {
            class_name: class_name.to_string(),
            name,
            descriptor,
            is_virtual,
        },
        // Declared-but-unnamed slot: still callable, as a no-op.
        _ => TrampEntry {
            class_name: class_name.to_string(),
            name: String::new(),
            descriptor: String::new(),
            is_virtual,
        },
    };
    let id = {
        let mut table = shared.trampolines.lock();
        table.push(entry);
        (table.len() - 1) as u32
    };
    core.make_svc_stub(SVC_CATEGORY_JAVA_TRAMPOLINE, id)
}

// ---- app class scan + registration (unchanged structure) ----

fn scan_class_headers(core: &ArmCore, data_start: u32, data_end: u32) -> Vec<u32> {
    let in_data = |v: u32| v >= data_start && v < data_end;
    let is_short_name = |ptr: u32| -> bool {
        if ptr == 0 {
            return false;
        }
        match read_null_terminated_string_bytes(core, ptr) {
            Ok(b) => !b.is_empty() && b.len() <= 24 && b.iter().all(|&c| (0x20..0x7f).contains(&c)),
            Err(_) => false,
        }
    };
    let small_count = |table: u32| -> bool {
        if table == 0 {
            return true;
        }
        if !in_data(table) {
            return false;
        }
        matches!(read_generic::<u32, _>(core, table), Ok(c) if c < 512)
    };

    let mut out = Vec::new();
    let mut va = data_start;
    while va + 0x40 <= data_end {
        let read = |off: u32| read_generic::<u32, _>(core, va + off).unwrap_or(0);
        let tag = read(0);
        let parent_ok = read(0x10) == 0 || in_data(read(0x10)) || is_short_name(read(0x10));
        if tag > 0 && tag < 0x1000 && is_short_name(read(0x08)) && parent_ok && small_count(read(0x38)) && small_count(read(0x3c)) {
            out.push(va);
        }
        va += 4;
    }
    out
}

/// Scan the app's `.data` for native class headers and register each as an
/// ARM-backed JVM class. No-op (empty) when none are found (clet path unaffected).
pub async fn register_app_classes(jvm: &Jvm, core: &mut ArmCore, shared: &LgtJvmShared, data_start: u32, data_end: u32) -> Result<Vec<String>> {
    let headers = scan_class_headers(core, data_start, data_end);
    if headers.is_empty() {
        return Ok(Vec::new());
    }
    tracing::debug!("LGT native JVM: found {} app class headers in .data", headers.len());

    let mut pending: Vec<LgtNativeClass> = Vec::new();
    let mut seen = BTreeSet::new();
    for header in headers {
        if let Ok(class) = parse_native_class(core, header)
            && !class.name.is_empty()
            && seen.insert(class.name.clone())
        {
            pending.push(class);
        }
    }
    let app_names: BTreeSet<String> = pending.iter().map(|c| c.name.clone()).collect();

    // Compute each app class's instance-field object slots (inherited-first flat
    // layout). Stored for `java_load_classes` to fill `field_offsets`. See cp17.
    *shared.app_field_layouts.lock() = compute_field_layouts(&pending);

    let mut registered = Vec::new();
    let mut done = BTreeSet::new();
    loop {
        let mut progressed = false;
        let mut still = Vec::new();
        for class in pending {
            let parent_ready = match &class.parent_name {
                Some(p) => !app_names.contains(p) || done.contains(p),
                None => true,
            };
            if !parent_ready {
                still.push(class);
                continue;
            }
            let name = class.name.clone();
            let definition = LgtClassDefinition::from_native(&class, core.clone(), shared.clone());
            match jvm.register_class(Box::new(definition), None).await {
                Ok(_) => {
                    tracing::debug!("LGT native JVM: registered {name:?} (parent={:?})", class.parent_name);
                    done.insert(name.clone());
                    registered.push(name);
                    progressed = true;
                }
                Err(e) => tracing::warn!("LGT native JVM: failed to register {name:?}: {e:?}"),
            }
        }
        pending = still;
        if pending.is_empty() || !progressed {
            break;
        }
    }
    if !pending.is_empty() {
        let names: Vec<&String> = pending.iter().map(|c| &c.name).collect();
        tracing::warn!("LGT native JVM: {} classes left unregistered: {names:?}", pending.len());
    }
    Ok(registered)
}

/// Compute each app class's instance-field object slots using the inherited-first
/// flat guest layout: `slot = (field count of all app ancestors) + declared index`.
/// Platform ancestors terminate the chain contributing 0 (their fields live JVM-side,
/// not in the guest field array). Pure over the parsed class set.
///
/// Why inherited-first: the AOT addresses an instance field as
/// `obj.field[field_offsets[K]]` into one flat per-object array, laying a subclass's
/// own fields *after* every inherited field. Mapping each ref to this slot de-aliases
/// the otherwise all-zero `field_offsets` table. See `docs/lgt_abi.md` §5.
fn compute_field_layouts(classes: &[LgtNativeClass]) -> AppFieldLayouts {
    // (parent_name, field_count) by class name.
    let meta: BTreeMap<String, (Option<String>, u32)> = classes
        .iter()
        .map(|c| (c.name.clone(), (c.parent_name.clone(), c.fields.len() as u32)))
        .collect();
    let mut layouts: AppFieldLayouts = Vec::with_capacity(classes.len());
    for c in classes {
        // app-ancestor field count (platform parents end the chain at 0)
        let mut base = 0u32;
        let mut name = c.parent_name.clone();
        while let Some(n) = name {
            match meta.get(&n) {
                Some((parent, count)) => {
                    base += count;
                    name = parent.clone();
                }
                None => break,
            }
        }
        let fields = c
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.type_descriptor.clone(), base + f.index))
            .collect();
        layouts.push((c.name.clone(), fields));
    }
    layouts
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;
    use alloc::vec;
    use alloc::vec::Vec;

    use super::super::native_class::LgtNativeField;
    use super::{
        LgtNativeClass, VTABLE_REFS, char_array_data_size, compute_field_layouts, known_java_lang_vtable, physical_vtable_slot,
        write_char_array_block,
    };

    fn cls(name: &str, parent: Option<&str>, fields: &[(&str, &str, u32)]) -> LgtNativeClass {
        LgtNativeClass {
            ptr: 0,
            tag: 0x21,
            name: name.into(),
            ptr_parent: 0,
            parent_name: parent.map(Into::into),
            access_flags: 0,
            methods: vec![],
            fields: fields
                .iter()
                .map(|(n, t, i)| LgtNativeField {
                    ptr_class: 0,
                    name: (*n).into(),
                    type_descriptor: (*t).into(),
                    access_flags: 0,
                    index: *i,
                })
                .collect(),
        }
    }

    /// Instance fields use an inherited-first flat layout: `slot = (field count of all
    /// app ancestors) + declared index`, platform parents terminating the chain at 0.
    /// Mirrors the 150/150 reference-app fields validated at runtime (cp17); locked
    /// here with a deterministic hierarchy (`a`→`Game`, `o`→`d`/`e`).
    #[test]
    fn field_layout_inherited_first() {
        let classes = vec![
            cls("a", None, &[("appA", "I", 0), ("appB", "I", 1)]),
            cls("Game", Some("a"), &[("g", "I", 0)]),
            cls("o", Some("org/kwis/msp/lcdui/Card"), &[("f0", "I", 0), ("f1", "I", 1), ("f2", "I", 2)]),
            cls("d", Some("o"), &[("d0", "I", 0), ("d1", "I", 1)]),
            cls("e", Some("o"), &[("e0", "I", 0)]),
        ];
        let layouts = compute_field_layouts(&classes);
        let slot = |cn: &str, fname: &str| -> u32 {
            let (_, fl) = layouts.iter().find(|(n, _)| n == cn).unwrap();
            fl.iter().find(|(n, _, _)| n == fname).unwrap().2
        };
        // platform-terminated chain (`o` extends Card → base 0)
        assert_eq!(slot("o", "f0"), 0);
        assert_eq!(slot("o", "f2"), 2);
        // `a` is a root app class (base 0)
        assert_eq!(slot("a", "appA"), 0);
        assert_eq!(slot("a", "appB"), 1);
        // `Game` inherits a's 2 fields → base 2
        assert_eq!(slot("Game", "g"), 2);
        // `d`/`e` inherit o's 3 fields → base 3 (siblings share the base)
        assert_eq!(slot("d", "d0"), 3);
        assert_eq!(slot("d", "d1"), 4);
        assert_eq!(slot("e", "e0"), 3);
    }

    /// Reserved-slot-0 two-level dispatch (cp15): install writes ref `r` at
    /// `physical_vtable_slot(r)`; the AOT reads `vtable[virtual_method_offsets[r] + 1]`
    /// with `virtual_method_offsets[r] = r`. The two must coincide for every ref, never
    /// touch slot 0, and never collide.
    #[test]
    fn vtable_reserved_slot_zero() {
        assert_eq!(physical_vtable_slot(0), 1);
        let mut seen = BTreeSet::new();
        for r in 0..VTABLE_REFS {
            let install = physical_vtable_slot(r);
            let dispatch = r + 1; // vtable[offset[r] + 1], offset[r] = r
            assert_eq!(install, dispatch, "ref {r}: install/dispatch slot mismatch");
            assert!(install >= 1, "ref {r} must not use reserved slot 0");
            assert!(seen.insert(install), "ref {r} collides on physical slot {install}");
        }
    }

    /// Per-class override vtables: the AOT direct-dispatches a few `java/lang` classes
    /// at hardcoded PHYSICAL slots (reserved slot already baked in).
    #[test]
    fn per_class_override_slots() {
        assert_eq!(
            known_java_lang_vtable("java/lang/Runtime"),
            [(13u32, "freeMemory", "()J"), (14u32, "gc", "()V")].as_slice()
        );
        assert_eq!(known_java_lang_vtable("java/lang/Thread"), [(11u32, "start", "()V")].as_slice());
        let sb = known_java_lang_vtable("java/lang/StringBuffer");
        assert!(sb.iter().any(|&(i, n, _)| i == 5 && n == "toString"));
        assert!(sb.iter().any(|&(i, n, _)| i == 19 && n == "append"));
        // cp30: String dispatches char access via hardcoded physical slot 35 = toCharArray.
        assert_eq!(known_java_lang_vtable("java/lang/String"), [(35u32, "toCharArray", "()[C")].as_slice());
        // unknown classes get no override (they use the global identity vtable)
        assert!(known_java_lang_vtable("java/lang/Object").is_empty());
        assert!(known_java_lang_vtable("Game").is_empty());
    }

    /// cp31: the ez-i `char[]` guest layout — `{u32 len, u16 chars}` (chars at +4,
    /// stride 2, little-endian) — that the glyph loop @0x10228 reads. Locks the byte
    /// layout `materialize_char_array` writes.
    #[test]
    fn char_array_guest_layout() {
        use wie_core_arm::ArmCore;
        use wie_util::read_generic;

        // size = 4 (len) + 2 per char
        assert_eq!(char_array_data_size(0), 4);
        assert_eq!(char_array_data_size(10), 24);

        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000).unwrap();
        let data = 0x40000000u32;
        let chars: Vec<u16> = "LOADING...".encode_utf16().collect();
        assert_eq!(chars.len(), 10);
        write_char_array_block(&mut core, data, &chars).unwrap();

        // [data] = len (u32)
        assert_eq!(read_generic::<u32, _>(&core, data).unwrap(), 10);
        // [data + 4 + i*2] = char[i] (u16 LE)
        for (i, &ch) in chars.iter().enumerate() {
            assert_eq!(read_generic::<u16, _>(&core, data + 4 + i as u32 * 2).unwrap(), ch);
        }
        // 'L' = 0x4c, '.' = 0x2e
        assert_eq!(read_generic::<u16, _>(&core, data + 4).unwrap(), b'L' as u16);
        assert_eq!(read_generic::<u16, _>(&core, data + 4 + 9 * 2).unwrap(), b'.' as u16);
    }
}
