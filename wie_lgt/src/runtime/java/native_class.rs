//! Read-only parser for the LGT native class-descriptor format.
//!
//! ez-i AOT-compiled LGT apps emit each Java class as a record in the ELF `.data`
//! segment whose method bodies are native ARM code (pointers into `.text`). This
//! module decodes those records into plain structs for inspection — it does **not**
//! register anything with the JVM. See `docs/lgt_native_classes.md` for the full
//! byte layout and how it was reverse-engineered.
//!
//! Confirmed layout (from the ez-i reference app, all offsets little-endian u32
//! unless noted; 283/283 method code pointers validated inside `.text`):
//!
//! ```text
//! Class header (at H):
//!   +0x00  tag           (0x21 / 0x31 observed)
//!   +0x08  ptr_name      -> cstring (obfuscated, e.g. "Game", "i", "l")
//!   +0x10  ptr_parent    -> platform class-name cstring, OR another class record
//!   +0x18  access_flags  (java access bits | 0x20000 app marker)
//!   +0x38  ptr_methods   -> [count: u32, MethodRecord; count]
//!   +0x3c  ptr_fields    -> [count: u32, FieldRecord; count]
//!
//! Class handle (at H + 0x4c): { 0, 0, H }   <- members' `ptr_class` points here
//!
//! MethodRecord (28 bytes):
//!   +0x00 ptr_class (= handle)   +0x04 ptr_name      +0x08 ptr_signature
//!   +0x0c access_flags          +0x10 num_locals(?)  +0x14 code_ptr (-> .text)
//!   +0x18 unk
//!
//! FieldRecord (20 bytes):
//!   +0x00 ptr_class (= handle)   +0x04 ptr_name   +0x08 ptr_type
//!   +0x0c access_flags          +0x10 index/offset
//! ```

use alloc::string::String;
use alloc::vec::Vec;

use wie_core_arm::ArmCore;
use wie_util::{Result, read_generic, read_null_terminated_string_bytes};

const HDR_PTR_NAME: u32 = 0x08;
const HDR_PTR_PARENT: u32 = 0x10;
const HDR_ACCESS_FLAGS: u32 = 0x18;
const HDR_PTR_METHODS: u32 = 0x38;
const HDR_PTR_FIELDS: u32 = 0x3c;
/// Offset from a class header to its handle sub-structure (the `ptr_class` value
/// that field/method records carry).
#[allow(dead_code)]
pub const HDR_HANDLE_OFFSET: u32 = 0x4c;

const METHOD_RECORD_SIZE: u32 = 28;
const FIELD_RECORD_SIZE: u32 = 20;

#[derive(Debug, Clone)]
#[allow(dead_code)] // complete read-only parser API; not all fields consumed yet
pub struct LgtNativeMethod {
    pub ptr_class: u32,
    pub name: String,
    pub signature: String,
    pub access_flags: u32,
    pub num_locals: u32,
    /// Native ARM entry point (in `.text`).
    pub code_ptr: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // complete read-only parser API; not all fields consumed yet
pub struct LgtNativeField {
    pub ptr_class: u32,
    pub name: String,
    pub type_descriptor: String,
    pub access_flags: u32,
    pub index: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // complete read-only parser API; not all fields consumed yet
pub struct LgtNativeClass {
    /// Address of the class header record.
    pub ptr: u32,
    pub tag: u32,
    pub name: String,
    /// Raw parent pointer (`+0x10`): a platform class-name cstring or another
    /// class record. `parent_name` is the best-effort resolved name.
    pub ptr_parent: u32,
    pub parent_name: Option<String>,
    pub access_flags: u32,
    pub methods: Vec<LgtNativeMethod>,
    pub fields: Vec<LgtNativeField>,
}

fn read_cstring(core: &ArmCore, address: u32) -> Option<String> {
    if address == 0 {
        return None;
    }
    let bytes = read_null_terminated_string_bytes(core, address).ok()?;
    if bytes.is_empty() || !bytes.iter().all(|&b| (0x20..0x7f).contains(&b)) || bytes.len() > 64 {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// Parse the count-prefixed table at `table_ptr` using `stride`-byte records.
/// Guards against absurd counts (corrupt/misaligned pointers).
fn read_count(core: &ArmCore, table_ptr: u32) -> u32 {
    if table_ptr == 0 {
        return 0;
    }
    match read_generic::<u32, _>(core, table_ptr) {
        Ok(c) if c < 1024 => c,
        _ => 0,
    }
}

/// Parse a class header record at `ptr` into a [`LgtNativeClass`]. Read-only.
pub fn parse_native_class(core: &ArmCore, ptr: u32) -> Result<LgtNativeClass> {
    let tag = read_generic::<u32, _>(core, ptr)?;
    let ptr_name = read_generic::<u32, _>(core, ptr + HDR_PTR_NAME)?;
    let ptr_parent = read_generic::<u32, _>(core, ptr + HDR_PTR_PARENT)?;
    let access_flags = read_generic::<u32, _>(core, ptr + HDR_ACCESS_FLAGS)?;
    let ptr_methods = read_generic::<u32, _>(core, ptr + HDR_PTR_METHODS)?;
    let ptr_fields = read_generic::<u32, _>(core, ptr + HDR_PTR_FIELDS)?;

    let name = read_cstring(core, ptr_name).unwrap_or_default();

    // The parent reference (`+0x10`) is polymorphic:
    //   (a) a cstring pointer to a class name — platform classes
    //       ("org/kwis/msp/lcdui/Card", "java/lang/Object");
    //   (b) an app-class HANDLE (header + 0x4c) — e.g. Game -> class "a",
    //       d/e/j/l -> class "o"; the
    //       handle's +0x08 word is the parent header, whose +0x08 is its name;
    //   (c) (fallback) a class header directly.
    let parent_name = read_cstring(core, ptr_parent).or_else(|| {
        // A handle has the shape { 0, 0, header } -> resolve to its header; a bare
        // header has its name at +0x08.
        let is_handle = read_generic::<u32, _>(core, ptr_parent).ok() == Some(0) && read_generic::<u32, _>(core, ptr_parent + 4).ok() == Some(0);
        let header = if is_handle {
            read_generic::<u32, _>(core, ptr_parent + 8).ok()?
        } else {
            ptr_parent
        };
        let inner = read_generic::<u32, _>(core, header + HDR_PTR_NAME).ok()?;
        read_cstring(core, inner)
    });

    let method_count = read_count(core, ptr_methods);
    let mut methods = Vec::with_capacity(method_count as usize);
    for i in 0..method_count {
        let base = ptr_methods + 4 + i * METHOD_RECORD_SIZE;
        methods.push(LgtNativeMethod {
            ptr_class: read_generic(core, base)?,
            name: read_cstring(core, read_generic(core, base + 4)?).unwrap_or_default(),
            signature: read_cstring(core, read_generic(core, base + 8)?).unwrap_or_default(),
            access_flags: read_generic(core, base + 12)?,
            num_locals: read_generic(core, base + 16)?,
            code_ptr: read_generic(core, base + 20)?,
        });
    }

    let field_count = read_count(core, ptr_fields);
    let mut fields = Vec::with_capacity(field_count as usize);
    for i in 0..field_count {
        let base = ptr_fields + 4 + i * FIELD_RECORD_SIZE;
        fields.push(LgtNativeField {
            ptr_class: read_generic(core, base)?,
            name: read_cstring(core, read_generic(core, base + 4)?).unwrap_or_default(),
            type_descriptor: read_cstring(core, read_generic(core, base + 8)?).unwrap_or_default(),
            access_flags: read_generic(core, base + 12)?,
            index: read_generic(core, base + 16)?,
        });
    }

    Ok(LgtNativeClass {
        ptr,
        tag,
        name,
        ptr_parent,
        parent_name,
        access_flags,
        methods,
        fields,
    })
}

/// Resolve a class handle (`a0[i]`, i.e. header + 0x4c) back to its header, then
/// parse it. The handle's third word (`+0x08`) is the header pointer.
pub fn parse_native_class_from_handle(core: &ArmCore, handle: u32) -> Result<LgtNativeClass> {
    let header = read_generic::<u32, _>(core, handle + 8)?;
    parse_native_class(core, header)
}

#[cfg(test)]
mod tests {
    use wie_core_arm::ArmCore;
    use wie_util::{ByteWrite, Result, write_generic};

    use super::{parse_native_class, parse_native_class_from_handle};

    // Reference-app `.text` bounds (0x1000..0xe7800); method code pointers must fall
    // inside this range — the 283/283 in-`.text` invariant, here on a synthetic class.
    const TEXT_START: u32 = 0x1000;
    const TEXT_END: u32 = 0xe7800;

    /// Bump-allocating writer over a mapped guest region, for hand-encoding the ez-i
    /// class-descriptor byte format the parser consumes.
    struct Mem {
        core: ArmCore,
        cur: u32,
    }

    impl Mem {
        fn new() -> Result<Self> {
            let mut core = ArmCore::new(false, None)?;
            core.map(0x40000000, 0x4000)?;
            Ok(Self { core, cur: 0x40000000 })
        }
        fn align(&mut self) {
            self.cur = (self.cur + 3) & !3;
        }
        fn cstr(&mut self, s: &str) -> Result<u32> {
            self.align();
            let at = self.cur;
            let mut bytes = s.as_bytes().to_vec();
            bytes.push(0);
            self.core.write_bytes(at, &bytes)?;
            self.cur += bytes.len() as u32;
            Ok(at)
        }
        fn words(&mut self, ws: &[u32]) -> Result<u32> {
            self.align();
            let at = self.cur;
            for (i, w) in ws.iter().enumerate() {
                write_generic(&mut self.core, at + i as u32 * 4, *w)?;
            }
            self.cur += ws.len() as u32 * 4;
            Ok(at)
        }
        /// Zeroed 0x40-byte class header; caller patches the meaningful offsets.
        fn header(&mut self) -> Result<u32> {
            self.words(&[0u32; 16])
        }
        fn put(&mut self, base: u32, off: u32, val: u32) -> Result<()> {
            write_generic(&mut self.core, base + off, val)
        }
    }

    /// Parse a hand-encoded class descriptor and assert every header offset, the 28B
    /// method / 20B field record strides, the in-`.text` code-pointer invariant, class
    /// handle indirection (`from_handle`), and a parent declared via another class's
    /// handle (`d` → `o`).
    #[test]
    fn parse_descriptor_fixture() -> Result<()> {
        let mut m = Mem::new()?;

        // strings
        let name_o = m.cstr("o")?;
        let card = m.cstr("org/kwis/msp/lcdui/Card")?;
        let pn = m.cstr("paint")?;
        let ps = m.cstr("(Lorg/kwis/msp/lcdui/Graphics;)V")?;
        let un = m.cstr("update")?;
        let us = m.cstr("(I)V")?;
        let f0n = m.cstr("hp")?;
        let f0t = m.cstr("I")?;
        let f1n = m.cstr("mp")?;
        let f1t = m.cstr("I")?;
        let name_d = m.cstr("d")?;

        // method table: count=2, two 28-byte records
        //   [ptr_class, ptr_name, ptr_sig, access, num_locals, code_ptr, unk]
        let mtable = m.words(&[
            2, //
            0, pn, ps, 0x1, 3, 0x1f10, 0, //
            0, un, us, 0x1, 1, 0x2200, 0,
        ])?;
        // field table: count=2, two 20-byte records
        //   [ptr_class, ptr_name, ptr_type, access, index]
        let ftable = m.words(&[
            2, //
            0, f0n, f0t, 0x1, 0, //
            0, f1n, f1t, 0x1, 1,
        ])?;

        // class `o` header (parent = platform cstring "Card")
        let header_o = m.header()?;
        m.put(header_o, 0x00, 0x21)?; // tag
        m.put(header_o, 0x08, name_o)?;
        m.put(header_o, 0x10, card)?; // parent: cstring
        m.put(header_o, 0x18, 0x20021)?; // access (| 0x20000 app marker)
        m.put(header_o, 0x38, mtable)?;
        m.put(header_o, 0x3c, ftable)?;

        // class handle: { 0, 0, header }
        let handle_o = m.words(&[0, 0, header_o])?;

        // class `d` header (parent = the HANDLE of `o`, not a cstring)
        let empty_tbl = m.words(&[0])?; // count=0
        let header_d = m.header()?;
        m.put(header_d, 0x00, 0x21)?;
        m.put(header_d, 0x08, name_d)?;
        m.put(header_d, 0x10, handle_o)?; // parent: app-class handle
        m.put(header_d, 0x38, empty_tbl)?;
        m.put(header_d, 0x3c, empty_tbl)?;

        // --- assertions: header offsets + record strides ---
        let o = parse_native_class(&m.core, header_o)?;
        assert_eq!(o.tag, 0x21);
        assert_eq!(o.name, "o");
        assert_eq!(o.parent_name.as_deref(), Some("org/kwis/msp/lcdui/Card"));
        assert_eq!(o.access_flags, 0x20021);
        assert_eq!(o.methods.len(), 2);
        assert_eq!(o.methods[0].name, "paint");
        assert_eq!(o.methods[0].signature, "(Lorg/kwis/msp/lcdui/Graphics;)V");
        assert_eq!(o.methods[0].num_locals, 3);
        assert_eq!(o.methods[0].code_ptr, 0x1f10);
        assert_eq!(o.methods[1].name, "update");
        assert_eq!(o.methods[1].signature, "(I)V");
        assert_eq!(o.methods[1].code_ptr, 0x2200);
        assert_eq!(o.fields.len(), 2);
        assert_eq!(o.fields[0].name, "hp");
        assert_eq!(o.fields[0].type_descriptor, "I");
        assert_eq!(o.fields[0].index, 0);
        assert_eq!(o.fields[1].name, "mp");
        assert_eq!(o.fields[1].index, 1);

        // invariant: every method's native entry lies inside `.text`
        for mth in &o.methods {
            assert!(
                (TEXT_START..TEXT_END).contains(&mth.code_ptr),
                "method {} code_ptr {:#x} outside .text",
                mth.name,
                mth.code_ptr
            );
        }

        // class handle indirection resolves to the same class
        let via_handle = parse_native_class_from_handle(&m.core, handle_o)?;
        assert_eq!(via_handle.name, "o");
        assert_eq!(via_handle.methods.len(), 2);

        // parent declared via another class's handle resolves to its name
        let d = parse_native_class(&m.core, header_d)?;
        assert_eq!(d.name, "d");
        assert_eq!(d.parent_name.as_deref(), Some("o"));
        assert_eq!(d.methods.len(), 0);

        Ok(())
    }
}
