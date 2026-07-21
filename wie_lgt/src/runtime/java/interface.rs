use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use jvm::runtime::JavaLangString;
use wie_core_arm::{ArmCore, ResultWriter, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes};

use crate::runtime::java::native_jvm::{LgtJvmShared, install_platform_tables, register_app_classes};
use crate::runtime::wipi_c::invoke_lcdui_main;
use crate::runtime::{SVC_CATEGORY_INIT, SVC_CATEGORY_JAVA_INTERFACE, svc_ids::InitSvcId};

// LGT "java-interface" import module (table 0x64): the AOT-compiled app (ez-i / Xceed)
// registers itself with the platform through here and calls platform classes by
// resolved offset. See `docs/lgt_abi.md` §1 (thunk/dispatcher format), §3 (boot
// imports) and §6 (import table status).

/// Resolve java-interface import `function_index` (table `0x64`) to the SVC stub the
/// native code traps through. Boot imports get dedicated `SVC_CATEGORY_INIT` handlers;
/// everything else is routed by index through `SVC_CATEGORY_JAVA_INTERFACE` so the SVC
/// id *is* the import index (unimplemented indices are logged, not merged). See
/// `docs/lgt_abi.md` §1/§6.
pub fn get_java_interface_method(core: &mut ArmCore, function_index: u32) -> Result<u32> {
    Ok(match function_index {
        0x03 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk0)?,
        0x06 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk12)?,
        0x07 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk5)?,
        0x14 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaLoadClasses)?,
        0x82 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk9)?,
        0x83 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk11)?,
        // Native object allocator (`new`): `obj = java(0xf)(...); obj.<init>()`.
        0x0f => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaNewObject)?,
        // 0x54: method-entry/safepoint helper resolved first in every native method;
        // no-op (추정). See `java_interface_unk84`.
        0x54 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk84)?,
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
        // String factory: `(ctx, utf16_ptr=a1, count=a2, out_slot=a3)`. See the const.
        STRING_FACTORY_INDEX => java_new_string(core, shared, a1, a2).await?,
        GET_INSTANCE_INDEX => shared.singleton_instance(core, a0).await,
        SHOW_CARD_INDEX => {
            shared.show_card(a1).await?;
            0
        }
        LAZY_INSTANCE_INIT_INDEX => {
            shared.lazy_instance_init(core, a0, a1).await?;
            0
        }
        LAZY_CLASS_INIT_INDEX => {
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

/// Native String factory: `(ctx, utf16_ptr, count, out_slot)` → `java/lang/String`
/// from constant-pool UTF-16, called right before `StringBuffer.append`. See
/// `docs/lgt_abi.md` §6.
const STRING_FACTORY_INDEX: u32 = 0x9;

/// `getInstance(class_handle)` → a class's canonical singleton instance (must be stable
/// across calls/threads). See `docs/lgt_abi.md` §6.
const GET_INSTANCE_INDEX: u32 = 0xc;

/// `show-card` (Display.setCurrent / pushCard equivalent): `(jlet, card_guest, jlet)`.
/// See `docs/lgt_abi.md` §7 and `LgtJvmShared::show_card`.
const SHOW_CARD_INDEX: u32 = 0x57;

/// Lazy instance init: `0xd(instance, init_fn)` runs an instance's deferred initialiser
/// once (AOT guard `field[0x10] != 5`). See `docs/lgt_abi.md` §6.
const LAZY_INSTANCE_INIT_INDEX: u32 = 0xd;

/// Lazy class init: `0xb(class)` marks a class initialised (AOT guard
/// `[[class+8]+0x1a] != 3`). See `docs/lgt_abi.md` §6.
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

/// Import `0x07`: the app hands over its OWN class registry (`a0`: `[0]`=count,
/// `[2..]`=class handles, each `class_header + 0x4c`). This is the authoritative list
/// of the app's classes; register them here, ahead of `load_classes` (0x14) and
/// `Main.main` (0x83). `a1` is a trailing per-class byte array (role unconfirmed). See
/// `docs/lgt_abi.md` §3.
pub async fn java_unk5(core: &mut ArmCore, shared: &mut LgtJvmShared, a0: u32, a1: u32) -> Result<()> {
    tracing::debug!("java_unk5: app registry @ {a0:#x} (aux @ {a1:#x})");

    let jvm = shared.jvm.clone();
    let registered = register_app_classes(&jvm, core, shared, a0).await?;
    if !registered.is_empty() {
        tracing::info!("LGT native JVM: registered {} app classes: {registered:?}", registered.len());
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
    // Import `0x14`: declares the imported platform classes and the method/field refs
    // they use, and points at the writable `*_offsets` tables the platform fills so the
    // native code can dispatch into them. `install_platform_tables` builds the global
    // vtable + per-class overrides + instance field layout. The unused input/output
    // arrays below are not consumed yet (the resolved method/field tables suffice). See
    // `docs/lgt_abi.md` §4–5.
    let _ = (static_fields, a4, static_field_offsets, a9);

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

/// Import `0x83` (invoke-static): `(class_name_ptr=a0, _=a1, argc=a2, argv_ptr=a3)`,
/// observed as `Main.main` with `argv[0]` = the app's main Jlet class ("Game"). Boots
/// the Jlet through the shared lcdui Main path (as the WIPI-C clet boot does). See
/// `docs/lgt_abi.md` §3.
pub async fn java_unk11(core: &mut ArmCore, shared: &mut LgtJvmShared, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
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
