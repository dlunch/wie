//! App-class registration from the `0x07` class registry, the inherited-first
//! field-layout computation, and the card-class graph derivation. See
//! `docs/lgt_abi.md` §3/§5/§7.

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};

use jvm::Jvm;

use wie_core_arm::ArmCore;
use wie_util::{Result, read_generic, read_null_terminated_string_bytes};

use super::AppFieldLayouts;
use super::class_model::LgtClassDefinition;
use super::shared::LgtJvmShared;
use crate::runtime::java::native_class::{LgtNativeClass, parse_native_class, parse_native_class_from_handle};

/// Register the app's own native classes as ARM-backed JVM classes, from the **union**
/// of two sources (dedup by class name):
///
/// 1. the `0x07` class registry at `registry_ptr` (its `a0`): `[0]` = handle count,
///    `[2..]` = `count` class handles (each `class_header + 0x4c`; the handle's `+0x08`
///    points back to the header), decoded with `parse_native_class_from_handle`;
/// 2. a heuristic scan of the ELF `.data` segment (`shared.data_range`) for class
///    headers, decoded with `parse_native_class`.
///
/// Why the union: on the real ez-i ROM the `0x07` registry is an **incomplete subset**
/// (e.g. 15 of 20 classes — it omits the main Jlet `Game`, the Jlet base `a`, and the
/// title-card subclasses `b`/`i`), so a registry-only path leaves `Main.main` unable to
/// find `Game` (and `show_card` unable to resolve the title card). The `.data` scan is
/// the complete source; the registry is kept as a precise, non-heuristic input that the
/// scan's heuristics could in principle miss. Their union registers every app class.
/// No-op (empty) when neither source yields a class (clet path unaffected). See
/// `docs/lgt_abi.md` §3.
pub async fn register_app_classes(jvm: &Jvm, core: &mut ArmCore, shared: &LgtJvmShared, registry_ptr: u32) -> Result<Vec<String>> {
    let mut pending: Vec<LgtNativeClass> = Vec::new();
    let mut seen = BTreeSet::new();

    // Source 1: the `0x07` registry handles.
    let count = read_generic::<u32, _>(core, registry_ptr).unwrap_or(0);
    let mut registry_classes = 0usize;
    if count != 0 && count < 1024 {
        for i in 0..count {
            let handle = match read_generic::<u32, _>(core, registry_ptr + 8 + i * 4) {
                Ok(h) => h,
                Err(_) => continue,
            };
            if let Ok(class) = parse_native_class_from_handle(core, handle)
                && !class.name.is_empty()
                && seen.insert(class.name.clone())
            {
                registry_classes += 1;
                pending.push(class);
            }
        }
    }

    // Source 2: the `.data` header scan (catches classes the registry omits).
    let data_range = *shared.data_range.lock();
    let mut scan_classes = 0usize;
    if let Some((data_start, data_end)) = data_range {
        for header in scan_class_headers(core, data_start, data_end) {
            if let Ok(class) = parse_native_class(core, header)
                && !class.name.is_empty()
                && seen.insert(class.name.clone())
            {
                scan_classes += 1;
                pending.push(class);
            }
        }
    }

    tracing::debug!(
        "LGT native JVM: app classes from registry @ {registry_ptr:#x} ({count} handles -> {registry_classes}) \u{222a} .data scan (+{scan_classes}) = {} unique",
        pending.len()
    );
    if pending.is_empty() {
        return Ok(Vec::new());
    }
    let app_names: BTreeSet<String> = pending.iter().map(|c| c.name.clone()).collect();

    // Compute each app class's instance-field object slots (inherited-first flat
    // layout). Stored for `java_load_classes` to fill `field_offsets`. See cp17.
    *shared.app_field_layouts.lock() = compute_field_layouts(&pending);
    // Record the class inheritance graph (name -> parent) for `resolve_card_class`.
    *shared.app_class_graph.lock() = pending.iter().map(|c| (c.name.clone(), c.parent_name.clone())).collect();

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

/// Scan the app's `.data` segment for native class-header records, returning the header
/// addresses. A header is recognised heuristically: a small non-zero tag, a short
/// printable `+0x08` name, a parent (`+0x10`) that is null / in `.data` / a short name,
/// and small method/field table counts (`+0x38`/`+0x3c`). The decoded classes are
/// unioned with the `0x07` registry in `register_app_classes` (the scan is the complete
/// source; see that fn). No-op (empty) for clet apps, whose `.data` holds no descriptors.
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

/// The unique deepest app subclass of the platform `org/kwis/msp/lcdui/Card` in the
/// app-class graph (`name -> parent`), or `None` if no app class extends `Card` or the
/// deepest depth is shared by more than one class. The shown card is known to be a
/// `Card`; this derives WHICH app card class it is from the registered class hierarchy
/// instead of assuming an obfuscated symbol name (reference app: `Card <- o <- b <- i`
/// ⇒ `i`). See `LgtJvmShared::resolve_card_class` and `docs/lgt_abi.md` §7.
pub(super) fn deepest_card_subclass(graph: &BTreeMap<String, Option<String>>) -> Option<String> {
    const CARD_BASE: &str = "org/kwis/msp/lcdui/Card";
    // Depth of `name`'s chain to the platform Card base (None if it never reaches it).
    let depth_to_card = |name: &str| -> Option<u32> {
        let mut cur = name;
        for d in 1..=64u32 {
            match graph.get(cur).and_then(|p| p.as_deref()) {
                Some(CARD_BASE) => return Some(d),
                Some(parent) if graph.contains_key(parent) => cur = parent,
                _ => return None,
            }
        }
        None
    };
    let depths: Vec<(u32, &String)> = graph.keys().filter_map(|n| depth_to_card(n).map(|d| (d, n))).collect();
    let max_depth = depths.iter().map(|(d, _)| *d).max()?;
    let mut at_max = depths.iter().filter(|(d, _)| *d == max_depth);
    let first = at_max.next()?;
    if at_max.next().is_some() {
        return None; // ambiguous deepest card subclass
    }
    Some(first.1.clone())
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;
    use alloc::vec;
    use alloc::vec::Vec;

    use super::super::class_model::{char_array_data_size, write_char_array_block};
    use super::super::dispatch::{VTABLE_REFS, known_java_lang_vtable, physical_vtable_slot};
    use super::{compute_field_layouts, deepest_card_subclass};
    use crate::runtime::java::native_class::{LgtNativeClass, LgtNativeField};

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

    /// The shown card's app class is derived from the registered class graph as the
    /// unique deepest subclass of the platform `Card` base — not an assumed symbol name.
    /// Reference-app shape: `Card <- o <- {b,d,e,j,l}`, `b <- i` ⇒ `i` is uniquely the
    /// deepest. Non-Card app classes (`a`, `Game`) and ambiguity are excluded.
    #[test]
    fn derive_card_class_from_graph() {
        use alloc::collections::BTreeMap;
        use alloc::string::{String, ToString};

        let card = || Some("org/kwis/msp/lcdui/Card".to_string());
        let app = |p: &str| Some(p.to_string());
        let graph: BTreeMap<String, Option<String>> = [
            ("a".to_string(), None),
            ("Game".to_string(), app("a")),
            ("o".to_string(), card()),
            ("b".to_string(), app("o")),
            ("d".to_string(), app("o")),
            ("e".to_string(), app("o")),
            ("j".to_string(), app("o")),
            ("l".to_string(), app("o")),
            ("i".to_string(), app("b")),
        ]
        .into_iter()
        .collect();
        assert_eq!(deepest_card_subclass(&graph).as_deref(), Some("i"));

        // No app class extends Card -> None.
        let no_card: BTreeMap<String, Option<String>> = [("a".to_string(), None), ("Game".to_string(), app("a"))].into_iter().collect();
        assert_eq!(deepest_card_subclass(&no_card), None);

        // Two classes tie for deepest -> ambiguous -> None.
        let tie: BTreeMap<String, Option<String>> = [
            ("o".to_string(), card()),
            ("b".to_string(), app("o")),
            ("i".to_string(), app("b")),
            ("k".to_string(), app("b")),
        ]
        .into_iter()
        .collect();
        assert_eq!(deepest_card_subclass(&tie), None);
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
