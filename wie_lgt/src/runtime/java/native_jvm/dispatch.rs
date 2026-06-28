//! native -> platform dispatch: per-method trampolines, the two-level vtable, the
//! `java/lang/*` per-class overrides, and `install_platform_tables` /
//! `register_java_trampoline_handler`. See `docs/lgt_abi.md` §4.

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString},
    vec::Vec,
};

use jvm::{ClassInstance, JavaError, JavaType, JavaValue, Jvm, runtime::JavaLangString};

use wie_core_arm::{Allocator, ArmCore, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use super::FIELD_ARRAY_WORDS;
use super::shared::LgtJvmShared;
use crate::runtime::SVC_CATEGORY_JAVA_TRAMPOLINE;

#[derive(Clone)]
pub(super) struct TrampEntry {
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
        // A native-`new`'d object whose class is compiled away (still in `pending_new`),
        // now the target of a direct hardcoded `vtable[N]` call the global by-name
        // vtable misroutes to a foreign platform method. RE proved the result is
        // discarded at this call pattern (`docs/lgt_abi.md` §6, cp14), so pass it
        // through (return 0) rather than NPE. Scoped to `pending_new` so genuinely-bound
        // platform calls (which need a real value) never reach here.
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
pub(super) fn known_java_lang_vtable(class: &str) -> &'static [(u32, &'static str, &'static str)] {
    // Each arm's RE (which call site pinned each slot) is in `docs/lgt_abi.md` §4/§7.
    match class {
        "java/lang/Runtime" => &[(13, "freeMemory", "()J"), (14, "gc", "()V")],
        // append(String) is synthesised in the trampoline (wie's StringBuffer has only
        // append([CII)); see `handle_java_trampoline`.
        "java/lang/StringBuffer" => &[
            (5, "toString", "()Ljava/lang/String;"),
            (19, "append", "(Ljava/lang/String;)Ljava/lang/StringBuffer;"),
            (24, "append", "(I)Ljava/lang/StringBuffer;"),
        ],
        "java/lang/Thread" => &[(11, "start", "()V")],
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
pub(super) const VTABLE_REFS: u32 = 128;

/// Physical vtable word index for virtual method-ref `r`. The AOT dispatches via
/// `vtable[virtual_method_offsets[r] + 1]` (a literal `ldr ip,[r3,#4]` after
/// `add r3, r3, idx<<2`), so physical slot 0 is RESERVED and refs start at slot 1.
/// With `virtual_method_offsets[r] = r`, dispatch reads `vtable[r + 1]` — the slot
/// this returns. Centralising the `+1` keeps install and dispatch in lock-step (a
/// prior off-by-one here misrouted ref 6 to ref 7). See `docs/lgt_abi.md` §4.
pub(super) const fn physical_vtable_slot(r: u32) -> u32 {
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

    // 1) Global virtual vtable + identity index table. Physical slot 0 is reserved
    //    (`physical_vtable_slot`): ref `r` is installed at slot `r + 1` and
    //    `virtual_method_offsets[r] = r`, so the AOT's `vtable[offset[r] + 1]` dispatch
    //    lands on it. See `docs/lgt_abi.md` §4.
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
