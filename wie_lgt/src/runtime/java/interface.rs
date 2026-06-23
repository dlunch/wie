use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use jvm::runtime::JavaLangString;
use wie_core_arm::{ArmCore, ResultWriter, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes};

use crate::runtime::java::native_jvm::{LgtJvmShared, install_platform_tables};
use crate::runtime::wipi_c::invoke_lcdui_main;
use crate::runtime::{SVC_CATEGORY_INIT, SVC_CATEGORY_JAVA_INTERFACE, svc_ids::InitSvcId};

// LGT "java-interface" import module (table 0x64). The native application is an
// AOT-compiled Java program (ez-i / Xceed toolchain): its classes are emitted as
// native ARM code that registers itself with the platform through this module and
// calls platform classes (`org/kwis/...`, `java/...`) by resolved offset.
//
// Decoded import indices (see `get_java_interface_method`):
//   0x03 -> java_unk0          register main-class metadata (name, args)
//   0x06 -> java_unk12         (paired with 0x07; takes the same struct ptr)
//   0x07 -> java_unk5          register the app's OWN classes (native methods)
//   0x14 -> java_load_classes  declare IMPORTED platform classes + resolve offsets
//   0x82 -> java_unk9          (boot hook, arg always 0)
//   0x83 -> java_unk11         invoke-static org/kwis/msp/lcdui/Main.main(argv)

/// Resolve java-interface import `function_index` (table `0x64`) to the SVC stub the
/// native code will call.
///
/// How an app reaches here: each import site is a 16-byte thunk (`str lr; bl
/// <dispatcher>; .word table; .word index`). On first use the dispatcher asks the
/// platform to resolve `(table=0x64, index)` — which lands in this function — and the
/// returned SVC stub is what subsequent calls trap through. Boot imports get dedicated
/// `SVC_CATEGORY_INIT` handlers; everything else is routed by index through
/// `SVC_CATEGORY_JAVA_INTERFACE` so each import keeps its identity (the SVC id *is* the
/// index) and unimplemented ones are logged rather than silently merged. See
/// `docs/lgt_abi.md` §1 for the thunk/dispatcher format and §6 for the import table.
pub fn get_java_interface_method(core: &mut ArmCore, function_index: u32) -> Result<u32> {
    Ok(match function_index {
        0x03 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk0)?,
        0x06 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk12)?,
        0x07 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk5)?,
        0x14 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaLoadClasses)?,
        0x82 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk9)?,
        0x83 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk11)?,
        // Native object allocator (`new`): `obj = java(0xf)(...); obj.<init>()`. Returns
        // a guest object the <init> trampoline binds to a JVM instance of its class.
        0x0f => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaNewObject)?,
        // Runtime helper resolved lazily during native method execution (called
        // first in every method with a small per-method constant — looks like a
        // method-entry / stack-check / safepoint helper). Stubbed as a no-op.
        0x54 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk84)?,
        // Every other java-interface import is routed by its index (the SVC id *is*
        // the import index) so it keeps its identity: it is logged with its index and
        // specific imports (e.g. the native String factory) are implemented in
        // `handle_java_interface_svc`. Unimplemented indices return 0 (no-op).
        _ => core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, function_index)?,
    })
}

/// Dispatch for java-interface imports routed by `function_index` (the SVC id).
/// Reads the four ARM argument registers, runs the import, and writes the result.
pub async fn handle_java_interface_svc(core: &mut ArmCore, shared: &mut LgtJvmShared, id: SvcId) -> Result<()> {
    let (_, lr) = core.read_pc_lr()?;
    let index = id.0;
    let (a0, a1, a2, a3) = (core.read_param(0)?, core.read_param(1)?, core.read_param(2)?, core.read_param(3)?);

    let result = match index {
        // Native String factory: build a `java/lang/String` from UTF-16 constant data
        // and hand the native code a guest pointer bound to that JVM instance.
        //   a0 = context (string-pool/runtime handle, unused here)
        //   a1 = pointer to UTF-16 char data (the constant's length prefix already skipped)
        //   a2 = char count
        //   a3 = output slot (.bss) the native code also stashes the result in
        // Confirmed by RE: `func@0x1834` reads `const[idx] = {len:u16, char[len]:u16}`
        // then calls this import to materialise the String passed to StringBuffer.append.
        i if i == STRING_FACTORY_INDEX => java_new_string(core, shared, a1, a2).await?,
        // `getInstance`: return the singleton instance of the class whose descriptor
        // handle is `a0`. The AOT's `getInstance` (`func@0x18ac`) calls this and
        // dereferences the result as an object (`obj.field[..]`); it must be stable
        // across calls/threads so per-class state (e.g. the `a.run` run-flag at
        // `obj+0x20`) is shared. Identified cp20: `getInstance = import_0xc(class_handle,
        // registry)`; left as a no-op it returned 0, so the run-flag was never shared
        // and the game loop self-gated off.
        i if i == GET_INSTANCE_INDEX => shared.singleton_instance(core, a0).await,
        // `show-card` (Display.setCurrent / pushCard equivalent): the app hands the
        // platform a card guest block to display (`a0=jlet, a1=card, a2=jlet`). Rebind
        // it to the app card class and push it to wie's Display so the MIDP paint loop
        // ticks `o.paint` each frame (cp39). See `LgtJvmShared::show_card`.
        i if i == SHOW_CARD_INDEX => {
            shared.show_card(a1).await?;
            0
        }
        // Lazy instance init (cp51): `0xd(instance, init_fn)` runs the instance's
        // initialiser if it hasn't run yet (guarded on `field[0x10] != 5`). No-op'd,
        // getInstance singletons stayed uninitialised (empty fields → empty scene).
        // See `LgtJvmShared::lazy_instance_init`.
        i if i == LAZY_INSTANCE_INIT_INDEX => {
            shared.lazy_instance_init(core, a0, a1).await?;
            0
        }
        // Lazy class init (cp51): `0xb(class)` marks a class initialised so the AOT's
        // `if [[class+8]+0x1a] != 3` guard stops re-firing (it spun 3665× while no-op'd).
        i if i == LAZY_CLASS_INIT_INDEX => {
            shared.lazy_class_init(core, a0).await?;
            0
        }
        _ => {
            tracing::debug!("LGT java-interface import {index:#x}({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x}) lr={lr:#x} -> 0 (no-op)");
            0
        }
    };

    result.write(core, lr)
}

/// java-interface import index of the native String factory (UTF-16 chars -> String).
/// Identified empirically (cp10): import `0x9` is invoked as
/// `(0x1400154, <.text UTF-16 ptr>, <char count>, <.bss out slot>)` immediately
/// before `StringBuffer.append`, and the char data matches the constant pool (e.g.
/// `0xe7512`,len 4 = "txt/"). `func@0x1834` reads `const[idx]={len,chars}` then calls
/// this import to materialise the String.
const STRING_FACTORY_INDEX: u32 = 0x9;

/// java-interface import index of `getInstance(class_handle)` — returns a class's
/// canonical singleton instance. Identified cp20 (the `func@0x18ac` getInstance path
/// calls `import_0xc(class_handle, registry)` and dereferences the result).
const GET_INSTANCE_INDEX: u32 = 0xc;

/// java-interface import index of `show-card` (Display.setCurrent / pushCard
/// equivalent). Identified cp39 from the `a.run` trace: `0x57(jlet, card_guest, jlet)`
/// hands the platform the card the app `new`'d (the title card, guest `0x48840120`).
/// Left as a no-op before cp39, so the card was never pushed and `o.paint` never ran.
const SHOW_CARD_INDEX: u32 = 0x57;

/// java-interface import index of lazy instance init (cp51): `0xd(instance, init_fn)`
/// runs an instance's deferred initialiser the first time it's used (guarded by the AOT
/// on `field[0x10] != 5`). Identified by the call shape in the getInstance helper
/// `@0x1c304` (`if [inst+8].h[0x10] != 5 { r1 = init_fn; 0xd(inst, init_fn) }`).
const LAZY_INSTANCE_INIT_INDEX: u32 = 0xd;

/// java-interface import index of lazy class init (cp51): `0xb(class)` ensures a class is
/// initialised the first time it's used (AOT guard `if [[class+8]+0x1a] != 3`).
const LAZY_CLASS_INIT_INDEX: u32 = 0xb;

/// Materialise a `java/lang/String` from `count` UTF-16 chars at guest `chars_ptr`,
/// register it in the instance map behind a fresh guest proxy block, and return the
/// guest pointer (so it round-trips back to the JVM String when used as an argument).
async fn java_new_string(core: &mut ArmCore, shared: &mut LgtJvmShared, chars_ptr: u32, count: u32) -> Result<u32> {
    let mut units = Vec::with_capacity(count as usize);
    for i in 0..count {
        units.push(read_generic::<u16, _>(core, chars_ptr + i * 2)?);
    }
    let text = String::from_utf16_lossy(&units);
    tracing::debug!("java-interface String factory: {count} chars -> {text:?}");

    let jvm = shared.jvm.clone();
    let instance = match JavaLangString::from_rust_string(&jvm, &text).await {
        Ok(s) => s,
        Err(e) => return Err(JvmSupport::to_wie_err(&jvm, e).await),
    };
    Ok(shared.register_platform_object(core, instance))
}

// ---- memory-decode helpers (best-effort, never fail) ----

/// Read a null-terminated C string at `address` as a Rust String (lossy).
fn read_cstring(core: &ArmCore, address: u32) -> Option<String> {
    if address == 0 {
        return None;
    }
    let bytes = read_null_terminated_string_bytes(core, address).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// Read `count` u32 words starting at `address` (stops early on a read error).
fn peek_words(core: &ArmCore, address: u32, count: usize) -> Vec<u32> {
    let mut out = Vec::with_capacity(count);
    let mut cursor = address;
    for _ in 0..count {
        match read_generic::<u32, _>(core, cursor) {
            Ok(v) => out.push(v),
            Err(_) => break,
        }
        cursor += 4;
    }
    out
}

pub async fn java_unk0(core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    // (main_class_name_ptr, params_ptr, flag_str_ptr) — e.g. ("Game", _, "true")
    tracing::debug!(
        "java_unk0(main_class={:?}, {a1:#x}, flag={:?})",
        read_cstring(core, a0),
        read_cstring(core, a2)
    );

    Ok(())
}

pub async fn java_unk5(core: &mut ArmCore, _: &mut LgtJvmShared, a0: u32, a1: u32) -> Result<()> {
    // a0: the application's OWN native class registry.
    //   [0]    = handle count
    //   [1]    = 0
    //   [2..]  = `count` class HANDLES (each = class_header + 0x4c); the handle's
    //            +0x08 word points back to the class header record in `.data`.
    //   [2+n..]= trailing per-class byte array (small counts; role unconfirmed).
    // Each class record carries native method/field tables whose method bodies are
    // ARM code pointers (`.text`). See docs/lgt_native_classes.md and native_class.rs.
    //
    // This handler is diagnostic only (decode + trace). The actual registration of
    // the app's native classes as JVM classes is done by `register_app_classes`,
    // which scans the `.data` segment directly during `load_native` (it does not rely
    // on this registry pointer). See `docs/lgt_abi.md` §3.
    let count = read_generic::<u32, _>(core, a0).unwrap_or(0);
    tracing::debug!("java_unk5: app registry @ {a0:#x} ({count} class handles, aux @ {a1:#x})");

    if tracing::enabled!(tracing::Level::DEBUG) {
        for i in 0..count.min(64) {
            let handle = match read_generic::<u32, _>(core, a0 + 8 + i * 4) {
                Ok(h) => h,
                Err(_) => continue,
            };
            match crate::runtime::java::native_class::parse_native_class_from_handle(core, handle) {
                Ok(class) => {
                    tracing::debug!(
                        "  class[{i}] {:?} (tag={:#x} access={:#x}) parent={:?} methods={} fields={}",
                        class.name,
                        class.tag,
                        class.access_flags,
                        class.parent_name,
                        class.methods.len(),
                        class.fields.len()
                    );
                    for m in class.methods.iter().take(3) {
                        tracing::debug!("      {}{} code={:#x} locals={}", m.name, m.signature, m.code_ptr, m.num_locals);
                    }
                }
                Err(e) => tracing::debug!("  class[{i}] handle={handle:#x} parse failed: {e}"),
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn java_load_classes(
    core: &mut ArmCore,
    shared: &mut LgtJvmShared,
    classes: u32,
    fields: u32,
    static_fields: u32,
    virtual_methods: u32,
    a4: u32,
    static_methods: u32,
    field_offsets: u32,
    static_field_offsets: u32,
    virtual_method_offsets: u32,
    a9: u32,
    static_method_offsets: u32,
) -> Result<()> {
    // Declares the platform classes the app imports and resolves the layout the
    // native code uses to dispatch into them. Inputs:
    //   classes        = LgtJavaImportedClass[count] (count-prefixed); each entry is
    //                    { ptr_name, _, static_field_off/cnt, virtual_method_off/cnt,
    //                      _, static_method_off/cnt } (24 bytes).
    //   fields/static_fields/virtual_methods/a4/static_methods = arrays of
    //                    { ptr_name, ptr_type } pairs the imported classes reference.
    // Outputs (writable app RAM, e.g. 0x15006f4): the platform fills *_offsets with the
    //   resolved indices / vtable offsets so the native code can call platform methods.
    //   `install_platform_tables` (below) builds the global vtable + per-class
    //   overrides + instance field layout that fill these. See `docs/lgt_abi.md` §4–5.
    // The remaining input arrays (field-type pairs / alternate views) and the static
    // output tables are not consumed yet (the resolved method/field tables suffice).
    let _ = (static_fields, a4, static_field_offsets, a9);

    // Fill the native -> platform method/field offset tables.
    install_platform_tables(
        core,
        shared,
        classes,
        fields,
        virtual_methods,
        static_methods,
        field_offsets,
        virtual_method_offsets,
        static_method_offsets,
    )
}

pub async fn java_unk9(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::debug!("java_unk9({a0:#x})");

    Ok(())
}

pub async fn java_unk11(core: &mut ArmCore, shared: &mut LgtJvmShared, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    // Decoded calling convention (LGT java-interface import 0x83 — invoke-static):
    //   a0 = ptr to class name cstring  (observed: "org/kwis/msp/lcdui/Main")
    //   a1 = 0 (unused / implicit method "main")
    //   a2 = argc
    //   a3 = ptr to argv (array of `argc` cstring pointers)
    // argv[0] is the application's main Jlet class name (e.g. "Game"). This mirrors
    // the WIPI-C clet boot, which invokes the same Main.main with "net/wie/CletWrapper".
    let _ = a1;
    let class_name = read_cstring(core, a0).unwrap_or_default();
    let argc = a2 as usize;
    let argv_ptrs = peek_words(core, a3, argc.min(16));
    let argv: Vec<String> = argv_ptrs.iter().map(|&p| read_cstring(core, p).unwrap_or_default()).collect();

    tracing::debug!("java_unk11: invoke-static {class_name}.main argv={argv:?}");

    if class_name != "org/kwis/msp/lcdui/Main" {
        return Err(WieError::Unimplemented(format!(
            "LGT java_unk11: unexpected invoke target {class_name} (argv={argv:?})"
        )));
    }
    let main_class = argv.first().cloned().unwrap_or_default();
    if main_class.is_empty() {
        return Err(WieError::FatalError("LGT java_unk11: empty main class name in argv[0]".into()));
    }

    // Boot the application's main Jlet through the shared lcdui Main path.
    let mut jvm = shared.jvm.clone();
    invoke_lcdui_main(&mut jvm, &main_class).await
}

pub async fn java_unk12(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::debug!("java_unk12({a0:#x})");

    Ok(())
}

/// java-interface import `0x54`: runtime helper resolved lazily at the start of
/// native method execution. Treated as a no-op (returns 0) until its semantics are
/// confirmed. Appears benign (a stack/safepoint check).
pub async fn java_interface_unk84(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::trace!("java_interface_unk84({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(0)
}

/// Generic no-op stub for not-yet-implemented java-interface imports (returns 0).
pub async fn java_interface_stub(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<u32> {
    tracing::debug!("java_interface_stub({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x}) -> 0");

    Ok(0)
}
