use alloc::{string::String, vec::Vec};

use serde::Deserialize;

use super::{
    CaptureName, Entry, PatternToken,
    hook::{Hook, HookKind, InlineCopy, PatternHook, PatternHookKind},
    patch::{PatchSpec, PatternPatchSpec},
};

const BINARY_PATCHES_TOML: &str = include_str!("../../../data/binary_patches.toml");

pub fn binary_patches() -> Vec<Entry> {
    let doc: RawDoc = toml::from_str(BINARY_PATCHES_TOML).expect("parse data/binary_patches.toml");
    doc.entry.into_iter().map(RawEntry::into_entry).collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDoc {
    entry: Vec<RawEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    hash: Option<String>,
    name: String,
    #[serde(default)]
    hook: Vec<RawHook>,
    #[serde(default)]
    patch: Vec<RawPatch>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHook {
    kind: KindTag,
    pc: Option<u32>,
    pattern: Option<String>,
    dst_offset: Option<i32>,
    src_offset: Option<i32>,
    len_offset: Option<i32>,
    exit_pc: Option<u32>,
    spill_back: Option<bool>,
    count_offset: Option<i32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPatch {
    pc: Option<u32>,
    pattern: Option<String>,
    bytes: String,
    expect: Option<String>,
    offset: Option<u32>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum KindTag {
    Memcpy,
    Memset,
    Strcpy,
    Strlen,
    InlineCopy,
    RegInlineCopy,
}

impl RawEntry {
    fn into_entry(self) -> Entry {
        let name = self.name;
        let hash = self.hash.as_deref().map(|s| parse_hash(s, &name));
        let mut hooks = Vec::new();
        let mut hook_patterns = Vec::new();
        for raw in self.hook {
            match (raw.pc, raw.pattern.as_deref()) {
                (Some(pc), None) => hooks.push(Hook {
                    pc,
                    kind: pc_kind(&raw, &name),
                }),
                (None, Some(pat)) => {
                    let tokens = parse_pattern(pat, &name);
                    let kind_template = pattern_template(&raw, &tokens, &name);
                    hook_patterns.push(PatternHook { tokens, kind_template });
                }
                (Some(_), Some(_)) => panic!("entry {name}: hook cannot specify both `pc` and `pattern`"),
                (None, None) => panic!("entry {name}: hook must specify either `pc` or `pattern`"),
            }
        }
        if hash.is_none() && !hooks.is_empty() {
            panic!("entry {name}: hash is required when pc-based hooks are present (a pc only makes sense for a specific binary)");
        }
        let mut patches = Vec::new();
        let mut patch_patterns = Vec::new();
        for raw in self.patch {
            match into_patch_kind(raw, hash.is_some(), &name) {
                ParsedPatch::Pc(spec) => patches.push(spec),
                ParsedPatch::Pattern(spec) => patch_patterns.push(spec),
            }
        }
        Entry {
            hash,
            name,
            hooks,
            hook_patterns,
            patches,
            patch_patterns,
        }
    }
}

enum ParsedPatch {
    Pc(PatchSpec),
    Pattern(PatternPatchSpec),
}

fn into_patch_kind(raw: RawPatch, has_hash: bool, entry_name: &str) -> ParsedPatch {
    let bytes = parse_hex_bytes(&raw.bytes, entry_name, "bytes");
    if bytes.is_empty() {
        panic!("entry {entry_name}: patch `bytes` must be at least 1 byte");
    }
    reject_svc_pattern(&bytes, entry_name);
    let expect = raw.expect.as_deref().map(|s| {
        let v = parse_hex_bytes(s, entry_name, "expect");
        if v.len() != bytes.len() {
            panic!(
                "entry {entry_name}: patch `expect` length ({}) must equal `bytes` length ({})",
                v.len(),
                bytes.len()
            );
        }
        v
    });

    match (raw.pc, raw.pattern.as_deref()) {
        (Some(_), Some(_)) => panic!("entry {entry_name}: patch cannot specify both `pc` and `pattern`"),
        (None, None) => panic!("entry {entry_name}: patch must specify either `pc` or `pattern`"),
        (Some(pc), None) => {
            if !has_hash {
                panic!("entry {entry_name}: patch `pc` requires entry `hash` (a pc only makes sense for a specific binary)");
            }
            if raw.offset.is_some() {
                panic!("entry {entry_name}: patch `offset` is meaningless with `pc` (only valid for `pattern`)");
            }
            ParsedPatch::Pc(PatchSpec { pc, bytes, expect })
        }
        (None, Some(pat)) => {
            let tokens = parse_pattern(pat, entry_name);
            reject_patch_capture_tokens(&tokens, entry_name);
            let offset = raw.offset.unwrap_or(0);
            let pat_len = tokens.len() as u32;
            let bytes_len = bytes.len() as u32;
            let end = offset
                .checked_add(bytes_len)
                .unwrap_or_else(|| panic!("entry {entry_name}: patch `offset` ({offset}) + `bytes` length ({bytes_len}) overflows u32"));
            if end > pat_len {
                panic!("entry {entry_name}: patch `offset` ({offset}) + `bytes` length ({bytes_len}) exceeds pattern length ({pat_len})");
            }
            ParsedPatch::Pattern(PatternPatchSpec {
                tokens,
                bytes,
                expect,
                offset,
            })
        }
    }
}

fn parse_hex_bytes(s: &str, entry_name: &str, field: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for tok in s.split_whitespace() {
        if tok.len() != 2 || !tok.chars().all(|c| c.is_ascii_hexdigit()) {
            panic!("entry {entry_name}: patch `{field}` token `{tok}` is not a 2-char hex byte");
        }
        out.push(u8::from_str_radix(tok, 16).unwrap());
    }
    out
}

/// Reject `bytes` that, written as a 16-bit little-endian halfword, would
/// produce `SVC #0x80` — the same opcode the hook dispatcher patches in.
/// Allowing it would let a patch silently install an unregistered hook PC and
/// crash with a misleading "fired at unregistered PC" fatal at runtime.
fn reject_svc_pattern(bytes: &[u8], entry_name: &str) {
    for w in bytes.windows(2) {
        if w == [0x80, 0xdf] {
            panic!(
                "entry {entry_name}: patch `bytes` may not contain the SVC #0x80 instruction (`80 df` LE) — that would shadow the hook dispatcher"
            );
        }
    }
}

/// Patches do not get to consume capture results, so capture tokens in a patch
/// pattern have no place to land. Reject them at parse time to avoid a dead
/// match pattern that silently behaves like `??`.
fn reject_patch_capture_tokens(tokens: &[PatternToken], entry_name: &str) {
    for t in tokens {
        match t {
            PatternToken::Capture(_) => {
                panic!("entry {entry_name}: patch pattern may not contain `{{...}}` capture tokens (use `??` for wildcards)");
            }
            PatternToken::BitMatch { capture: Some(_), .. } => {
                panic!("entry {entry_name}: patch pattern may not contain register-capture bits in BitMatch (use `?` for wildcards)");
            }
            _ => {}
        }
    }
}

fn pc_kind(raw: &RawHook, entry_name: &str) -> HookKind {
    match raw.kind {
        KindTag::Memcpy => HookKind::Memcpy,
        KindTag::Memset => HookKind::Memset,
        KindTag::Strcpy => HookKind::Strcpy,
        KindTag::Strlen => HookKind::Strlen,
        KindTag::InlineCopy => HookKind::InlineCopy(InlineCopy {
            dst_offset: raw
                .dst_offset
                .unwrap_or_else(|| panic!("entry {entry_name}: pc-based inline_copy requires dst_offset")),
            src_offset: raw
                .src_offset
                .unwrap_or_else(|| panic!("entry {entry_name}: pc-based inline_copy requires src_offset")),
            len_offset: raw
                .len_offset
                .unwrap_or_else(|| panic!("entry {entry_name}: pc-based inline_copy requires len_offset")),
            exit_pc: raw
                .exit_pc
                .unwrap_or_else(|| panic!("entry {entry_name}: pc-based inline_copy requires exit_pc")),
            spill_back: raw
                .spill_back
                .unwrap_or_else(|| panic!("entry {entry_name}: inline_copy requires spill_back")),
        }),
        KindTag::RegInlineCopy => panic!("entry {entry_name}: reg_inline_copy must be pattern-based, not pc-based"),
    }
}

fn pattern_template(raw: &RawHook, tokens: &[PatternToken], entry_name: &str) -> PatternHookKind {
    match raw.kind {
        KindTag::Memcpy => PatternHookKind::Memcpy,
        KindTag::Memset => PatternHookKind::Memset,
        KindTag::Strcpy => PatternHookKind::Strcpy,
        KindTag::Strlen => PatternHookKind::Strlen,
        KindTag::RegInlineCopy => {
            let need = |cap: CaptureName, label: &str| {
                if !tokens.iter().any(|t| match t {
                    PatternToken::BitMatch { capture: Some((c, _)), .. } => *c == cap,
                    _ => false,
                }) {
                    panic!("entry {entry_name}: reg_inline_copy pattern must capture {label} register");
                }
            };
            need(CaptureName::SrcReg, "src");
            need(CaptureName::DstReg, "dst");
            need(CaptureName::CountReg, "count");
            PatternHookKind::RegInlineCopy {
                count_offset: raw
                    .count_offset
                    .unwrap_or_else(|| panic!("entry {entry_name}: reg_inline_copy requires count_offset")),
            }
        }
        KindTag::InlineCopy => {
            let exit_cap = tokens.iter().any(|t| matches!(t, PatternToken::Capture(CaptureName::ExitB)));
            if !exit_cap && raw.exit_pc.is_none() {
                panic!("entry {entry_name}: inline_copy pattern needs either {{exit_b}} capture or exit_pc");
            }
            if exit_cap && raw.exit_pc.is_some() {
                panic!("entry {entry_name}: inline_copy pattern cannot specify both {{exit_b}} and exit_pc");
            }
            PatternHookKind::InlineCopy {
                dst_offset: resolve_offset("dst_offset", tokens, CaptureName::Dst, raw.dst_offset, entry_name),
                src_offset: resolve_offset("src_offset", tokens, CaptureName::Src, raw.src_offset, entry_name),
                len_offset: resolve_offset("len_offset", tokens, CaptureName::Len, raw.len_offset, entry_name),
                exit_pc: raw.exit_pc,
                spill_back: raw
                    .spill_back
                    .unwrap_or_else(|| panic!("entry {entry_name}: inline_copy requires spill_back")),
            }
        }
    }
}

fn resolve_offset(field: &str, tokens: &[PatternToken], cap: CaptureName, fixed: Option<i32>, entry_name: &str) -> Option<i32> {
    let has_cap = tokens.iter().any(|t| matches!(t, PatternToken::Capture(c) if *c == cap));
    match (has_cap, fixed) {
        (true, None) => None,
        (false, Some(_)) => fixed,
        (true, Some(_)) => panic!("entry {entry_name}: {field} cannot be set when a corresponding capture is in the pattern"),
        (false, None) => panic!("entry {entry_name}: {field} required when no matching capture is in the pattern"),
    }
}

fn parse_hash(s: &str, entry_name: &str) -> [u8; 16] {
    if s.len() != 32 {
        panic!("entry {entry_name}: hash must be 32 hex chars (got {} chars: `{s}`)", s.len());
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .unwrap_or_else(|_| panic!("entry {entry_name}: hash contains non-hex byte at offset {i}: `{s}`"));
    }
    out
}

fn parse_pattern(pattern: &str, entry_name: &str) -> Vec<PatternToken> {
    let mut tokens = Vec::new();
    for raw in pattern.split_whitespace() {
        let token = if raw == "??" {
            PatternToken::AnyByte
        } else if raw.len() == 10
            && let Some(bits) = raw.strip_prefix("0b")
        {
            parse_bit_match(bits, entry_name)
        } else if let Some(rest) = raw.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            let cap = match rest {
                "dst" => CaptureName::Dst,
                "src" => CaptureName::Src,
                "len" => CaptureName::Len,
                "exit_b" => CaptureName::ExitB,
                _ => panic!("entry {entry_name}: unknown capture name {{{rest}}} (allowed: dst, src, len, exit_b)"),
            };
            PatternToken::Capture(cap)
        } else if raw.len() == 2 && raw.chars().all(|c| c.is_ascii_hexdigit()) {
            PatternToken::Literal(u8::from_str_radix(raw, 16).unwrap())
        } else {
            panic!("entry {entry_name}: invalid pattern token `{raw}`");
        };
        tokens.push(token);
    }
    validate_exit_b(&tokens, entry_name);
    tokens
}

fn validate_exit_b(tokens: &[PatternToken], entry_name: &str) {
    let mut pair_seen = false;
    let mut i = 0;
    while i < tokens.len() {
        if matches!(tokens[i], PatternToken::Capture(CaptureName::ExitB)) {
            let next_is_exit = matches!(tokens.get(i + 1), Some(PatternToken::Capture(CaptureName::ExitB)));
            if !next_is_exit {
                panic!("entry {entry_name}: {{exit_b}} must appear as two consecutive tokens");
            }
            if matches!(tokens.get(i + 2), Some(PatternToken::Capture(CaptureName::ExitB))) {
                panic!("entry {entry_name}: {{exit_b}} appears more than twice consecutively");
            }
            if pair_seen {
                panic!("entry {entry_name}: pattern may contain at most one {{exit_b}} pair");
            }
            pair_seen = true;
            i += 2;
        } else {
            i += 1;
        }
    }
}

/// Parse an 8-character byte specification of `0`/`1` literals, `?` wildcards,
/// and `s`/`d`/`c` register placeholders (3 consecutive of the same letter).
/// e.g. `00sss011` → mask=0b11000111, fixed=0b00000011, capture (src @ shift 3).
fn parse_bit_match(bits: &str, entry_name: &str) -> PatternToken {
    if bits.len() != 8 {
        panic!("entry {entry_name}: bit pattern `0b{bits}` must be 8 characters");
    }
    let mut mask: u8 = 0;
    let mut fixed: u8 = 0;
    let mut capture: Option<(CaptureName, u8, u8)> = None; // (name, lowest bit, count seen)
    for (i, ch) in bits.chars().enumerate() {
        let bit = 7 - i as u8;
        match ch {
            '0' => {
                mask |= 1 << bit;
            }
            '1' => {
                mask |= 1 << bit;
                fixed |= 1 << bit;
            }
            '?' => {}
            's' | 'd' | 'c' => {
                let name = match ch {
                    's' => CaptureName::SrcReg,
                    'd' => CaptureName::DstReg,
                    _ => CaptureName::CountReg,
                };
                match &mut capture {
                    Some((n, lowest, count)) if *n == name => {
                        *lowest = bit; // iterating high→low, so the latest write is the lowest bit
                        *count += 1;
                    }
                    Some(_) => panic!("entry {entry_name}: bit pattern `0b{bits}` mixes multiple register placeholders"),
                    None => capture = Some((name, bit, 1)),
                }
            }
            _ => panic!("entry {entry_name}: invalid char {ch:?} in bit pattern `0b{bits}` (allowed: 0,1,?,s,d,c)"),
        }
    }
    let capture = capture.map(|(name, lowest, count)| {
        if count != 3 {
            panic!("entry {entry_name}: bit pattern `0b{bits}` register placeholder must span exactly 3 bits");
        }
        (name, lowest)
    });
    PatternToken::BitMatch { mask, fixed, capture }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn parse_doc(toml_text: &str) -> Vec<Entry> {
        let doc: RawDoc = toml::from_str(toml_text).expect("parse test toml");
        doc.entry.into_iter().map(RawEntry::into_entry).collect()
    }

    #[test]
    fn pc_patch_with_expect_and_bytes() {
        let entries = parse_doc(
            r#"
            [[entry]]
            hash = "00000000000000000000000000000000"
            name = "pc-patch"

            [[entry.patch]]
            pc = 0x12345
            bytes = "00 20 70 47"
            expect = "78 47 01 22"
            "#,
        );
        let e = &entries[0];
        assert_eq!(e.patches.len(), 1);
        let p = &e.patches[0];
        assert_eq!(p.pc, 0x12345);
        assert_eq!(p.bytes, vec![0x00, 0x20, 0x70, 0x47]);
        assert_eq!(p.expect.as_deref(), Some([0x78, 0x47, 0x01, 0x22].as_slice()));
        assert!(e.patch_patterns.is_empty());
    }

    #[test]
    fn pattern_patch_with_offset_and_expect() {
        let entries = parse_doc(
            r#"
            [[entry]]
            name = "pattern-patch"

            [[entry.patch]]
            pattern = "aa bb cc dd"
            bytes = "11 22"
            expect = "bb cc"
            offset = 1
            "#,
        );
        let e = &entries[0];
        assert!(e.patches.is_empty());
        assert_eq!(e.patch_patterns.len(), 1);
        let pp = &e.patch_patterns[0];
        assert_eq!(pp.tokens.len(), 4);
        assert_eq!(pp.bytes, vec![0x11, 0x22]);
        assert_eq!(pp.expect.as_deref(), Some([0xbb, 0xcc].as_slice()));
        assert_eq!(pp.offset, 1);
    }

    #[test]
    fn pattern_patch_default_offset_is_zero() {
        let entries = parse_doc(
            r#"
            [[entry]]
            name = "no-offset"

            [[entry.patch]]
            pattern = "aa bb"
            bytes = "11 22"
            "#,
        );
        assert_eq!(entries[0].patch_patterns[0].offset, 0);
    }

    #[test]
    fn legacy_hook_only_entry_parses_with_empty_patches() {
        let entries = parse_doc(
            r#"
            [[entry]]
            name = "legacy"

            [[entry.hook]]
            kind = "memcpy"
            pattern = "78 47 00 00 01 40 2d e9 03 00 52 e3"
            "#,
        );
        let e = &entries[0];
        assert_eq!(e.hook_patterns.len(), 1);
        assert!(e.hooks.is_empty());
        assert!(e.patches.is_empty());
        assert!(e.patch_patterns.is_empty());
    }

    #[test]
    #[should_panic(expected = "cannot specify both")]
    fn pc_and_pattern_both_panics() {
        parse_doc(
            r#"
            [[entry]]
            hash = "00000000000000000000000000000000"
            name = "x"
            [[entry.patch]]
            pc = 0x100
            pattern = "aa bb"
            bytes = "11 22"
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "must specify either")]
    fn pc_and_pattern_neither_panics() {
        parse_doc(
            r#"
            [[entry]]
            name = "x"
            [[entry.patch]]
            bytes = "11 22"
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "at least 1 byte")]
    fn empty_bytes_panics() {
        parse_doc(
            r#"
            [[entry]]
            hash = "00000000000000000000000000000000"
            name = "x"
            [[entry.patch]]
            pc = 0x100
            bytes = ""
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "must equal `bytes` length")]
    fn bytes_expect_length_mismatch_panics() {
        parse_doc(
            r#"
            [[entry]]
            hash = "00000000000000000000000000000000"
            name = "x"
            [[entry.patch]]
            pc = 0x100
            bytes = "11 22"
            expect = "33"
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "requires entry `hash`")]
    fn pc_patch_without_hash_panics() {
        parse_doc(
            r#"
            [[entry]]
            name = "x"
            [[entry.patch]]
            pc = 0x100
            bytes = "11 22"
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "overflows u32")]
    fn offset_plus_bytes_u32_overflow_panics() {
        // offset 0xffff_ffff + bytes_len 2 wraps before the > pat_len check.
        parse_doc(
            r#"
            [[entry]]
            name = "x"
            [[entry.patch]]
            pattern = "aa bb"
            bytes = "11 22"
            offset = 4294967295
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "exceeds pattern length")]
    fn offset_plus_bytes_overflow_panics() {
        parse_doc(
            r#"
            [[entry]]
            name = "x"
            [[entry.patch]]
            pattern = "aa bb"
            bytes = "11 22"
            offset = 1
            "#,
        );
    }

    #[test]
    fn offset_plus_bytes_exactly_at_pattern_end_is_ok() {
        let entries = parse_doc(
            r#"
            [[entry]]
            name = "x"
            [[entry.patch]]
            pattern = "aa bb cc dd"
            bytes = "11 22"
            offset = 2
            "#,
        );
        assert_eq!(entries[0].patch_patterns[0].offset, 2);
    }

    #[test]
    #[should_panic(expected = "meaningless with `pc`")]
    fn pc_with_offset_panics() {
        parse_doc(
            r#"
            [[entry]]
            hash = "00000000000000000000000000000000"
            name = "x"
            [[entry.patch]]
            pc = 0x100
            bytes = "11 22"
            offset = 1
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "may not contain the SVC #0x80")]
    fn bytes_containing_svc_80_panics() {
        parse_doc(
            r#"
            [[entry]]
            hash = "00000000000000000000000000000000"
            name = "x"
            [[entry.patch]]
            pc = 0x100
            bytes = "80 df"
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "may not contain `{...}` capture")]
    fn patch_pattern_with_capture_token_panics() {
        parse_doc(
            r#"
            [[entry]]
            name = "x"
            [[entry.patch]]
            pattern = "aa {dst} cc dd"
            bytes = "11 22"
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "may not contain register-capture bits")]
    fn patch_pattern_with_bitmatch_capture_panics() {
        parse_doc(
            r#"
            [[entry]]
            name = "x"
            [[entry.patch]]
            pattern = "0b00sss011 22 33 44"
            bytes = "11 22"
            "#,
        );
    }

    #[test]
    #[should_panic(expected = "unknown field")]
    fn patch_with_unknown_field_panics() {
        parse_doc(
            r#"
            [[entry]]
            hash = "00000000000000000000000000000000"
            name = "x"
            [[entry.patch]]
            pc = 0x100
            bytes = "11 22"
            kind = "memcpy"
            "#,
        );
    }
}
