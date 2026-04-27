use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde::Deserialize;

use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic};

use crate::{ArmCore, engine::ArmRegister, function::JumpTo, stdlib};

/// `hash` (when set) must equal the MD5 of the on-disk binary payload
/// (pre-relocation). `hash = None` installs unconditionally and is only valid
/// when every hook is pattern-based.
pub struct Entry {
    pub hash: Option<[u8; 16]>,
    pub name: String,
    pub hooks: Vec<Hook>,
    pub patterns: Vec<PatternHook>,
}

#[derive(Debug, Clone, Copy)]
pub struct Hook {
    /// LSB=1 (Thumb) is required; the engine doesn't service ARM-mode SVCs.
    pub pc: u32,
    pub kind: HookKind,
}

#[derive(Debug, Clone, Copy)]
pub enum HookKind {
    /// ABI: dst=r0, src=r1, len=r2; returns via LR.
    Memcpy,
    /// ABI: dst=r0, val=r1 (low byte), len=r2; returns via LR.
    Memset,
    /// ABI: dst=r0, src=r1; copies NUL-inclusive; returns via LR (R0 unchanged).
    Strcpy,
    /// ABI: str=r0; returns length in R0 via LR.
    Strlen,
    /// Replaces an inline byte-copy loop. Requires a down-counter `len` on the
    /// stack — zeroing it has to terminate the loop, so up-counter (`for i = 0;
    /// i < N`) shapes are not compatible.
    InlineCopy(InlineCopy),
}

#[derive(Debug, Clone, Copy)]
pub struct InlineCopy {
    pub dst_offset: i32,
    pub src_offset: i32,
    pub len_offset: i32,
    pub exit_pc: u32,
    /// Set when the loop's outer code re-reads the stack slots after the body;
    /// the dispatcher then writes back `dst+len`, `src+len`, `len=0`.
    pub spill_back: bool,
}

/// Scanned across the install-time memory range; each match becomes a `Hook`.
pub struct PatternHook {
    pub tokens: Vec<PatternToken>,
    pub kind_template: PatternHookKind,
}

pub enum PatternToken {
    Literal(u8),
    AnyByte,
    Capture(CaptureName),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CaptureName {
    Dst,
    Src,
    Len,
    ExitB,
}

pub enum PatternHookKind {
    Memcpy,
    Memset,
    Strcpy,
    Strlen,
    InlineCopy {
        /// `None` => filled from the matching `{dst}` / `{src}` / `{len}`
        /// capture (Thumb1 `SUBS Rn, #imm8` byte, negated to a stack offset).
        /// `Some(v)` => pinned by TOML when the pattern omits the capture.
        dst_offset: Option<i32>,
        src_offset: Option<i32>,
        len_offset: Option<i32>,
        /// `None` => filled from the `{exit_b}` capture's decoded branch target.
        exit_pc: Option<u32>,
        spill_back: bool,
    },
}

/// 0x80..0xFF sits above every category used by KTF/LGT/SKT (<=18) and still
/// fits in the 8-bit Thumb SVC immediate.
pub const NATIVE_HOOK_CATEGORY_BASE: u32 = 0x80;
pub const NATIVE_HOOK_MAX: usize = 0x80;

const NATIVE_HOOKS_TOML: &str = include_str!("../../data/native_hooks.toml");

pub fn native_hooks() -> Vec<Entry> {
    let doc: RawDoc = toml::from_str(NATIVE_HOOKS_TOML).expect("parse data/native_hooks.toml");
    doc.entry.into_iter().map(RawEntry::into_entry).collect()
}

#[derive(Deserialize)]
struct RawDoc {
    #[serde(default)]
    entry: Vec<RawEntry>,
}

#[derive(Deserialize)]
struct RawEntry {
    #[serde(default)]
    hash: Option<String>,
    name: String,
    #[serde(default)]
    hook: Vec<RawHook>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RawHook {
    Memcpy {
        #[serde(default)]
        pc: Option<u32>,
        #[serde(default)]
        pattern: Option<String>,
    },
    Memset {
        #[serde(default)]
        pc: Option<u32>,
        #[serde(default)]
        pattern: Option<String>,
    },
    Strcpy {
        #[serde(default)]
        pc: Option<u32>,
        #[serde(default)]
        pattern: Option<String>,
    },
    Strlen {
        #[serde(default)]
        pc: Option<u32>,
        #[serde(default)]
        pattern: Option<String>,
    },
    InlineCopy {
        #[serde(default)]
        pc: Option<u32>,
        #[serde(default)]
        pattern: Option<String>,
        #[serde(default)]
        dst_offset: Option<i32>,
        #[serde(default)]
        src_offset: Option<i32>,
        #[serde(default)]
        len_offset: Option<i32>,
        #[serde(default)]
        exit_pc: Option<u32>,
        spill_back: bool,
    },
}

impl RawEntry {
    fn into_entry(self) -> Entry {
        let hash = self.hash.as_deref().map(parse_hash);
        let mut hooks = Vec::new();
        let mut patterns = Vec::new();
        for raw in self.hook {
            match raw.split(&self.name) {
                Either::Left(hook) => hooks.push(hook),
                Either::Right(pattern) => patterns.push(pattern),
            }
        }
        if hash.is_none() && !hooks.is_empty() {
            panic!(
                "entry {}: hash is required when pc-based hooks are present (a pc only makes sense for a specific binary)",
                self.name
            );
        }
        Entry {
            hash,
            name: self.name,
            hooks,
            patterns,
        }
    }
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

impl RawHook {
    fn split(self, entry_name: &str) -> Either<Hook, PatternHook> {
        let (pc, pattern) = match &self {
            RawHook::Memcpy { pc, pattern }
            | RawHook::Memset { pc, pattern }
            | RawHook::Strcpy { pc, pattern }
            | RawHook::Strlen { pc, pattern }
            | RawHook::InlineCopy { pc, pattern, .. } => (*pc, pattern.clone()),
        };
        match (pc, pattern) {
            (Some(pc), None) => Either::Left(Hook {
                pc,
                kind: self.into_pc_kind(entry_name),
            }),
            (None, Some(pat)) => {
                let tokens = parse_pattern(&pat, entry_name);
                let kind_template = self.into_pattern_template(&tokens, entry_name);
                Either::Right(PatternHook { tokens, kind_template })
            }
            (Some(_), Some(_)) => panic!("entry {entry_name}: hook cannot specify both `pc` and `pattern`"),
            (None, None) => panic!("entry {entry_name}: hook must specify either `pc` or `pattern`"),
        }
    }

    fn into_pc_kind(self, entry_name: &str) -> HookKind {
        match self {
            RawHook::Memcpy { .. } => HookKind::Memcpy,
            RawHook::Memset { .. } => HookKind::Memset,
            RawHook::Strcpy { .. } => HookKind::Strcpy,
            RawHook::Strlen { .. } => HookKind::Strlen,
            RawHook::InlineCopy {
                dst_offset,
                src_offset,
                len_offset,
                exit_pc,
                spill_back,
                ..
            } => HookKind::InlineCopy(InlineCopy {
                dst_offset: dst_offset.unwrap_or_else(|| panic!("entry {entry_name}: pc-based inline_copy requires dst_offset")),
                src_offset: src_offset.unwrap_or_else(|| panic!("entry {entry_name}: pc-based inline_copy requires src_offset")),
                len_offset: len_offset.unwrap_or_else(|| panic!("entry {entry_name}: pc-based inline_copy requires len_offset")),
                exit_pc: exit_pc.unwrap_or_else(|| panic!("entry {entry_name}: pc-based inline_copy requires exit_pc")),
                spill_back,
            }),
        }
    }

    fn into_pattern_template(self, tokens: &[PatternToken], entry_name: &str) -> PatternHookKind {
        match self {
            RawHook::Memcpy { .. } => PatternHookKind::Memcpy,
            RawHook::Memset { .. } => PatternHookKind::Memset,
            RawHook::Strcpy { .. } => PatternHookKind::Strcpy,
            RawHook::Strlen { .. } => PatternHookKind::Strlen,
            RawHook::InlineCopy {
                dst_offset,
                src_offset,
                len_offset,
                exit_pc,
                spill_back,
                ..
            } => {
                let exit_cap = tokens.iter().any(|t| matches!(t, PatternToken::Capture(CaptureName::ExitB)));
                if !exit_cap && exit_pc.is_none() {
                    panic!("entry {entry_name}: inline_copy pattern needs either {{exit_b}} capture or exit_pc");
                }
                if exit_cap && exit_pc.is_some() {
                    panic!("entry {entry_name}: inline_copy pattern cannot specify both {{exit_b}} and exit_pc");
                }
                PatternHookKind::InlineCopy {
                    dst_offset: resolve_offset("dst_offset", tokens, CaptureName::Dst, dst_offset, entry_name),
                    src_offset: resolve_offset("src_offset", tokens, CaptureName::Src, src_offset, entry_name),
                    len_offset: resolve_offset("len_offset", tokens, CaptureName::Len, len_offset, entry_name),
                    exit_pc,
                    spill_back,
                }
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

fn parse_hash(s: &str) -> [u8; 16] {
    assert_eq!(s.len(), 32, "hash must be 32 hex chars: {s}");
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap_or_else(|_| panic!("invalid hex in hash: {s}"));
    }
    out
}

fn parse_pattern(pattern: &str, entry_name: &str) -> Vec<PatternToken> {
    let mut tokens = Vec::new();
    for raw in pattern.split_whitespace() {
        let token = if raw == "??" {
            PatternToken::AnyByte
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

#[derive(Debug, Clone)]
struct PatternMatch {
    addr: u32,
    dst: Option<u8>,
    src: Option<u8>,
    len: Option<u8>,
    /// Address of the first `{exit_b}` byte — combined with `exit_b_bytes` to
    /// compute the branch target.
    exit_b_site: Option<u32>,
    exit_b_bytes: Option<[u8; 2]>,
}

pub fn md5(data: &[u8]) -> [u8; 16] {
    md5::compute(data).0
}

/// Patches each hook site with an SVC and registers a per-id dispatcher.
/// `scan_ranges` are `(base, size)` byte ranges searched for pattern hooks
/// (typically the guest `.text` region). Returns the number of hooks
/// installed (duplicate-PC matches are skipped). Errors on ARM-mode PCs.
pub fn install(core: &mut ArmCore, entry: &Entry, scan_ranges: &[(u32, u32)]) -> Result<usize> {
    let mut installed: Vec<Hook> = entry.hooks.clone();

    for pattern in &entry.patterns {
        let matches = scan_pattern(core, pattern, scan_ranges)?;
        for (match_addr, pm) in matches {
            let kind = match &pattern.kind_template {
                PatternHookKind::Memcpy => HookKind::Memcpy,
                PatternHookKind::Memset => HookKind::Memset,
                PatternHookKind::Strcpy => HookKind::Strcpy,
                PatternHookKind::Strlen => HookKind::Strlen,
                PatternHookKind::InlineCopy {
                    dst_offset,
                    src_offset,
                    len_offset,
                    exit_pc,
                    spill_back,
                } => {
                    let dst = dst_offset
                        .or_else(|| pm.dst.map(capture_to_offset))
                        .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing dst")))?;
                    let src = src_offset
                        .or_else(|| pm.src.map(capture_to_offset))
                        .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing src")))?;
                    let len = len_offset
                        .or_else(|| pm.len.map(capture_to_offset))
                        .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing len")))?;
                    let exit = if let Some(v) = exit_pc {
                        *v
                    } else {
                        let site = pm
                            .exit_b_site
                            .ok_or_else(|| WieError::FatalError("pattern missing exit_b site".to_string()))?;
                        let bytes = pm
                            .exit_b_bytes
                            .ok_or_else(|| WieError::FatalError("pattern missing exit_b bytes".to_string()))?;
                        decode_exit_b(site, bytes)
                    };
                    HookKind::InlineCopy(InlineCopy {
                        dst_offset: dst,
                        src_offset: src,
                        len_offset: len,
                        exit_pc: exit,
                        spill_back: *spill_back,
                    })
                }
            };
            let pc = match_addr | 1;
            if installed.iter().any(|h| h.pc == pc) {
                tracing::warn!("Native hook at {pc:#x} already registered; skipping duplicate match");
                continue;
            }
            installed.push(Hook { pc, kind });
        }
    }

    if installed.len() > NATIVE_HOOK_MAX {
        return Err(WieError::FatalError(format!(
            "Native hook table for {} has {} entries, max {}",
            entry.name,
            installed.len(),
            NATIVE_HOOK_MAX
        )));
    }

    tracing::info!("Installing {} native hooks for {}", installed.len(), entry.name);

    for (i, hook) in installed.iter().enumerate() {
        if hook.pc & 1 == 0 {
            return Err(WieError::FatalError(format!(
                "Native hook PC {:#x} targets ARM mode; only Thumb (LSB=1) is supported",
                hook.pc
            )));
        }

        let category = NATIVE_HOOK_CATEGORY_BASE + i as u32;
        match hook.kind {
            HookKind::Memcpy => core.register_svc_handler(category, handle_memcpy, &())?,
            HookKind::Memset => core.register_svc_handler(category, handle_memset, &())?,
            HookKind::Strcpy => core.register_svc_handler(category, handle_strcpy, &())?,
            HookKind::Strlen => core.register_svc_handler(category, handle_strlen, &())?,
            HookKind::InlineCopy(spec) => core.register_svc_handler(category, handle_inline_copy, &spec)?,
        }

        let patch_addr = hook.pc & !1;
        let instruction: u16 = 0xdf00 | (category as u16 & 0xff);
        core.write_bytes(patch_addr, &instruction.to_le_bytes())?;

        tracing::info!("Native hook installed at {:#x}: {:?}", hook.pc, hook.kind);
    }

    Ok(installed.len())
}

/// Scans on 2-byte boundaries since all Thumb instructions are halfword-aligned.
fn scan_pattern(core: &mut ArmCore, pattern: &PatternHook, scan_ranges: &[(u32, u32)]) -> Result<Vec<(u32, PatternMatch)>> {
    let mut results = Vec::new();
    let pat_len = pattern.tokens.len();
    if pat_len == 0 {
        return Ok(results);
    }

    for (base, size) in scan_ranges {
        let mut buf = vec![0u8; *size as usize];
        core.read_bytes(*base, &mut buf)?;

        let mut off = 0usize;
        while off + pat_len <= buf.len() {
            if let Some(mut pm) = try_match(&pattern.tokens, &buf[off..off + pat_len]) {
                pm.addr = base + off as u32;
                if pm.exit_b_bytes.is_some() {
                    for (ti, t) in pattern.tokens.iter().enumerate() {
                        if matches!(t, PatternToken::Capture(CaptureName::ExitB)) {
                            pm.exit_b_site = Some(base + (off + ti) as u32);
                            break;
                        }
                    }
                }
                results.push((pm.addr, pm));
            }
            off += 2;
        }
    }

    Ok(results)
}

/// Returns a `PatternMatch` with captures populated; `addr` and `exit_b_site`
/// are filled in by the caller from the actual scan position.
fn try_match(tokens: &[PatternToken], bytes: &[u8]) -> Option<PatternMatch> {
    if bytes.len() < tokens.len() {
        return None;
    }
    let mut m = PatternMatch {
        addr: 0,
        dst: None,
        src: None,
        len: None,
        exit_b_site: None,
        exit_b_bytes: None,
    };
    let mut i = 0;
    while i < tokens.len() {
        let b = bytes[i];
        match &tokens[i] {
            PatternToken::Literal(v) => {
                if b != *v {
                    return None;
                }
                i += 1;
            }
            PatternToken::AnyByte => {
                i += 1;
            }
            PatternToken::Capture(name) => match name {
                CaptureName::Dst => {
                    m.dst = Some(b);
                    i += 1;
                }
                CaptureName::Src => {
                    m.src = Some(b);
                    i += 1;
                }
                CaptureName::Len => {
                    m.len = Some(b);
                    i += 1;
                }
                CaptureName::ExitB => {
                    if i + 1 >= bytes.len() || i + 1 >= tokens.len() {
                        return None;
                    }
                    let hi = bytes[i + 1];
                    if !matches!(tokens[i + 1], PatternToken::Capture(CaptureName::ExitB)) {
                        return None;
                    }
                    // Reject matches where the captured bytes aren't a Thumb
                    // unconditional `B imm11` (`11100 iiiiiiiiiii`, encoded as
                    // 0xE0xx-0xE7xx little-endian). Without this, a permissive
                    // pattern can land on arbitrary instructions and decode
                    // them as if they were branches, sending the dispatcher
                    // to a garbage `exit_pc`.
                    if hi & 0xf8 != 0xe0 {
                        return None;
                    }
                    m.exit_b_bytes = Some([b, hi]);
                    i += 2;
                }
            },
        }
    }
    Some(m)
}

/// Negate the unsigned `SUBS Rn, #imm8` immediate: the captured byte is the
/// distance below R7, so the resulting offset is `-imm8`.
fn capture_to_offset(byte: u8) -> i32 {
    -(byte as i32)
}

/// Decode a Thumb `B imm11` (`11100 iiiiiiiiiii`) at `b_site`. Returns the
/// target PC with the Thumb bit set.
fn decode_exit_b(b_site: u32, bytes: [u8; 2]) -> u32 {
    let raw = u16::from_le_bytes(bytes);
    let imm11 = (raw & 0x07ff) as i32;
    let offset = if imm11 & 0x400 != 0 { imm11 - 0x800 } else { imm11 };
    let target = (b_site.wrapping_add(4) as i64).wrapping_add((offset * 2) as i64) as u32;
    target | 1
}

async fn handle_memcpy(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, len: u32) -> Result<()> {
    tracing::trace!("native hook memcpy(ptr_dst={ptr_dst:#x}, ptr_src={ptr_src:#x}, len={len:#x})");
    stdlib::memcpy(core, ptr_dst, ptr_src, len)
}

async fn handle_memset(core: &mut ArmCore, _: &mut (), ptr_dst: u32, val: u32, len: u32) -> Result<()> {
    tracing::trace!("native hook memset(ptr_dst={ptr_dst:#x}, val={:#x}, len={len:#x})", val as u8);
    stdlib::memset(core, ptr_dst, val as u8, len)
}

async fn handle_strcpy(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32) -> Result<()> {
    tracing::trace!("native hook strcpy(ptr_dst={ptr_dst:#x}, ptr_src={ptr_src:#x})");
    stdlib::strcpy(core, ptr_dst, ptr_src)?;
    Ok(())
}

async fn handle_strlen(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    let len = stdlib::strlen(core, ptr_str)?;
    tracing::trace!("native hook strlen(ptr_str={ptr_str:#x}) -> {len:#x}");
    Ok(len)
}

async fn handle_inline_copy(core: &mut ArmCore, ctx: &mut InlineCopy) -> Result<JumpTo> {
    let r7 = {
        let inner = core.inner.lock();
        inner.engine.reg_read(ArmRegister::R7)
    };

    let dst_slot = r7.wrapping_add(ctx.dst_offset as u32);
    let src_slot = r7.wrapping_add(ctx.src_offset as u32);
    let len_slot = r7.wrapping_add(ctx.len_offset as u32);

    let ptr_dst: u32 = read_generic(core, dst_slot)?;
    let ptr_src: u32 = read_generic(core, src_slot)?;
    let len: u32 = read_generic(core, len_slot)?;

    tracing::trace!(
        "native hook inline_copy(ptr_dst={ptr_dst:#x}, ptr_src={ptr_src:#x}, len={len:#x}, exit={:#x})",
        ctx.exit_pc
    );
    stdlib::memcpy(core, ptr_dst, ptr_src, len)?;

    if ctx.spill_back {
        let new_ptr_dst = ptr_dst.wrapping_add(len);
        let new_ptr_src = ptr_src.wrapping_add(len);
        core.write_bytes(dst_slot, &new_ptr_dst.to_le_bytes())?;
        core.write_bytes(src_slot, &new_ptr_src.to_le_bytes())?;
        core.write_bytes(len_slot, &0u32.to_le_bytes())?;
    }

    Ok(JumpTo(ctx.exit_pc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::{RegisteredFunction, RegisteredFunctionHolder};

    #[test]
    fn embedded_toml_parses() {
        let entries = native_hooks();
        assert!(!entries.is_empty(), "embedded TOML produced no entries");
        for entry in &entries {
            assert!(!entry.name.is_empty(), "entry missing name");
        }
    }

    #[test]
    fn install_rejects_arm_mode_pc() -> Result<()> {
        let entry = Entry {
            hash: Some([0u8; 16]),
            name: "arm-mode".into(),
            hooks: vec![Hook {
                pc: 0x2000, // LSB=0 => ARM mode
                kind: HookKind::Memcpy,
            }],
            patterns: vec![],
        };
        let mut core = ArmCore::new(false)?;
        core.map(0x2000, 0x1000)?;

        let err = install(&mut core, &entry, &[]).unwrap_err();
        let msg = alloc::format!("{err}");
        assert!(msg.contains("ARM mode"), "unexpected error: {msg}");
        Ok(())
    }

    #[test]
    fn install_patches_thumb_svc_instruction() -> Result<()> {
        let entry = Entry {
            hash: Some([0u8; 16]),
            name: "patch".into(),
            hooks: vec![Hook {
                pc: 0x2001, // Thumb
                kind: HookKind::Memcpy,
            }],
            patterns: vec![],
        };
        let mut core = ArmCore::new(false)?;
        core.map(0x2000, 0x1000)?;

        core.write_bytes(0x2000, &[0xaa, 0xbb])?;

        install(&mut core, &entry, &[])?;

        let mut buf = [0u8; 2];
        core.read_bytes(0x2000, &mut buf)?;
        assert_eq!(buf, [NATIVE_HOOK_CATEGORY_BASE as u8, 0xdf]);
        Ok(())
    }

    #[futures_test::test]
    async fn memcpy_dispatch_copies_bytes_and_returns_via_lr() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        core.map(0x10000, 0x1000)?;

        let src = 0x10000u32;
        let dst = 0x10400u32;
        let data = [1u8, 2, 3, 4, 5, 6, 7, 8];
        core.write_bytes(src, &data)?;

        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, src);
            inner.engine.reg_write(ArmRegister::R2, data.len() as u32);
            inner.engine.reg_write(ArmRegister::LR, 0xdead_beef);
            inner.engine.reg_write(ArmRegister::PC, 0);
        }

        RegisteredFunctionHolder::new(handle_memcpy, &()).call(&mut core).await?;

        let mut out = [0u8; 8];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, data);

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), 0xdead_beef & !1);
        Ok(())
    }

    #[futures_test::test]
    async fn inline_copy_dispatch_reads_frame_copies_and_jumps_to_exit() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        core.map(0x10000, 0x2000)?;

        let src = 0x10000u32;
        let dst = 0x10800u32;
        let payload = [0xaa, 0xbb, 0xcc, 0xdd];
        core.write_bytes(src, &payload)?;

        let frame = 0x11000u32;
        core.write_bytes(frame, &dst.to_le_bytes())?;
        core.write_bytes(frame + 4, &src.to_le_bytes())?;
        core.write_bytes(frame + 8, &(payload.len() as u32).to_le_bytes())?;

        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R7, frame);
        }

        let ctx = InlineCopy {
            dst_offset: 0,
            src_offset: 4,
            len_offset: 8,
            exit_pc: 0x10401,
            spill_back: true,
        };
        RegisteredFunctionHolder::new(handle_inline_copy, &ctx).call(&mut core).await?;

        let mut out = [0u8; 4];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, payload);

        let mut slot = [0u8; 4];
        core.read_bytes(frame, &mut slot)?;
        assert_eq!(u32::from_le_bytes(slot), dst + payload.len() as u32);
        core.read_bytes(frame + 4, &mut slot)?;
        assert_eq!(u32::from_le_bytes(slot), src + payload.len() as u32);
        core.read_bytes(frame + 8, &mut slot)?;
        assert_eq!(u32::from_le_bytes(slot), 0);

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), 0x10400);
        assert_ne!(inner.engine.reg_read(ArmRegister::Cpsr) & 0x20, 0);
        Ok(())
    }

    #[futures_test::test]
    async fn install_then_execute_hits_dispatcher_end_to_end() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        core.map(0x20000, 0x2000)?;
        core.map(0x30000, 0x1000)?;

        let src = 0x30000u32;
        let dst = 0x30200u32;
        let payload = [9u8, 8, 7, 6];
        core.write_bytes(src, &payload)?;

        let hook_pc = 0x20001u32;
        let entry = Entry {
            hash: Some([0u8; 16]),
            name: "e2e".into(),
            hooks: vec![Hook {
                pc: 0x20001,
                kind: HookKind::Memcpy,
            }],
            patterns: vec![],
        };
        install(&mut core, &entry, &[])?;

        let mut opcode = [0u8; 2];
        core.read_bytes(0x20000, &mut opcode)?;
        assert_eq!(opcode[1], 0xdf);
        assert_eq!(opcode[0] as u32, NATIVE_HOOK_CATEGORY_BASE);

        let return_addr = 0x40000u32 | 1;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, src);
            inner.engine.reg_write(ArmRegister::R2, payload.len() as u32);
            inner.engine.reg_write(ArmRegister::LR, return_addr);
            inner.engine.reg_write(ArmRegister::PC, hook_pc);

            let cpsr = inner.engine.reg_read(ArmRegister::Cpsr);
            inner.engine.reg_write(ArmRegister::Cpsr, (cpsr & !0x3f) | 0x1f | 0x20);
            inner.engine.reg_write(ArmRegister::SP, 0x20f00);
        }

        let result = {
            let mut inner = core.inner.lock();
            inner.engine.run(0, 10)?
        };
        let category = match result {
            crate::engine::EngineRunResult::Svc { category, lr, spsr } => {
                let mut inner = core.inner.lock();
                inner.engine.reg_write(ArmRegister::Cpsr, spsr);
                inner.engine.reg_write(ArmRegister::PC, lr);
                category
            }
            _ => panic!("expected Svc"),
        };
        assert_eq!(category, NATIVE_HOOK_CATEGORY_BASE);

        let mut core_clone = core.clone();
        RegisteredFunctionHolder::new(handle_memcpy, &()).call(&mut core_clone).await?;

        let mut out = [0u8; 4];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, payload);

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), return_addr & !1);
        Ok(())
    }

    #[futures_test::test]
    async fn memset_dispatch_fills_bytes() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        core.map(0x10000, 0x1000)?;

        let dst = 0x10000u32;
        let len = 16u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, 0xAB);
            inner.engine.reg_write(ArmRegister::R2, len);
            inner.engine.reg_write(ArmRegister::LR, 0x3000);
        }

        RegisteredFunctionHolder::new(handle_memset, &()).call(&mut core).await?;

        let mut out = [0u8; 16];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, [0xabu8; 16]);
        Ok(())
    }

    #[futures_test::test]
    async fn strcpy_dispatch_copies_null_terminated_string_and_returns_via_lr() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        core.map(0x10000, 0x1000)?;

        let src = 0x10000u32;
        let dst = 0x10400u32;
        let s = b"hello, world!\0";
        core.write_bytes(src, s)?;

        core.write_bytes(dst, &[0xff; 32])?;

        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, src);
            inner.engine.reg_write(ArmRegister::LR, 0xcafe_babe);
            inner.engine.reg_write(ArmRegister::PC, 0);
        }

        RegisteredFunctionHolder::new(handle_strcpy, &()).call(&mut core).await?;

        let mut out = [0u8; 14];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(&out, s);

        let mut sentinel = [0u8; 1];
        core.read_bytes(dst + s.len() as u32, &mut sentinel)?;
        assert_eq!(sentinel, [0xff]);

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::R0), dst);
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), 0xcafe_babe & !1);
        Ok(())
    }

    #[futures_test::test]
    async fn strlen_dispatch_returns_length_in_r0() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        core.map(0x10000, 0x1000)?;

        let str_ptr = 0x10100u32;
        let s = b"abcdef\0";
        core.write_bytes(str_ptr, s)?;

        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, str_ptr);
            inner.engine.reg_write(ArmRegister::LR, 0x1234_5678);
            inner.engine.reg_write(ArmRegister::PC, 0);
        }

        RegisteredFunctionHolder::new(handle_strlen, &()).call(&mut core).await?;

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::R0), 6);
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), 0x1234_5678 & !1);
        Ok(())
    }

    #[test]
    fn pattern_scan_matches_single_hit() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        core.map(0x50000, 0x200)?;

        let pat_bytes = [0xaa, 0xbb, 0xcc, 0xdd];
        let match_addr = 0x50020u32;
        core.write_bytes(match_addr, &pat_bytes)?;

        let entry = Entry {
            hash: None,
            name: "scan-single".into(),
            hooks: vec![],
            patterns: vec![PatternHook {
                tokens: vec![
                    PatternToken::Literal(0xaa),
                    PatternToken::Literal(0xbb),
                    PatternToken::Literal(0xcc),
                    PatternToken::Literal(0xdd),
                ],
                kind_template: PatternHookKind::Memcpy,
            }],
        };

        install(&mut core, &entry, &[(0x50000, 0x200)])?;

        let mut out = [0u8; 2];
        core.read_bytes(match_addr, &mut out)?;
        assert_eq!(out[1], 0xdf);
        assert_eq!(out[0] as u32, NATIVE_HOOK_CATEGORY_BASE);
        Ok(())
    }

    #[test]
    fn pattern_scan_capture_extracts_dst_src() {
        let tokens = [
            PatternToken::Literal(0x00),
            PatternToken::Capture(CaptureName::Dst),
            PatternToken::Literal(0x3d),
            PatternToken::Capture(CaptureName::Src),
        ];
        let bytes = [0x00, 0x08, 0x3d, 0x0c];

        let m = try_match(&tokens, &bytes).expect("match");
        assert_eq!(m.dst, Some(0x08));
        assert_eq!(m.src, Some(0x0c));
        assert_eq!(capture_to_offset(0x08), -8);
        assert_eq!(capture_to_offset(0x0c), -12);
        // imm8 in `SUBS Rn, #imm8` is unsigned 0..=255, so high values must
        // negate to large negative offsets, not wrap through `i8`.
        assert_eq!(capture_to_offset(0x80), -128);
        assert_eq!(capture_to_offset(0xfc), -252);
    }

    #[test]
    fn pattern_scan_exit_b_computes_forward_branch_target() {
        let bytes = [0x02, 0xe0];
        let site = 0x100u32;
        let exit = decode_exit_b(site, bytes);
        assert_eq!(exit, (site + 4 + 4) | 1);

        let neg = decode_exit_b(site, [0xfe, 0xe7]);
        assert_eq!(neg, site | 1);
    }

    #[test]
    fn pattern_scan_rejects_exit_b_capture_when_bytes_are_not_thumb_b() {
        let tokens = [PatternToken::Capture(CaptureName::ExitB), PatternToken::Capture(CaptureName::ExitB)];
        // `0c 3a` is `SUBS R2, #0xC` — high byte 0x3a is not in 0xE0-0xE7.
        assert!(try_match(&tokens, &[0x0c, 0x3a]).is_none());
        // `e8 e7` is a valid back-branch (high byte 0xE7).
        assert!(try_match(&tokens, &[0xe8, 0xe7]).is_some());
    }

    #[test]
    fn pattern_duplicate_pc_warns_once_and_skips() -> Result<()> {
        let mut core = ArmCore::new(false)?;
        core.map(0x60000, 0x100)?;
        core.write_bytes(0x60010, &[0x11, 0x22, 0x33, 0x44])?;

        let entry = Entry {
            hash: None,
            name: "dup".into(),
            hooks: vec![],
            patterns: vec![
                PatternHook {
                    tokens: vec![PatternToken::Literal(0x11), PatternToken::Literal(0x22)],
                    kind_template: PatternHookKind::Memcpy,
                },
                PatternHook {
                    tokens: vec![PatternToken::Literal(0x11), PatternToken::Literal(0x22)],
                    kind_template: PatternHookKind::Memcpy,
                },
            ],
        };
        let count = install(&mut core, &entry, &[(0x60000, 0x100)])?;
        assert_eq!(count, 1, "duplicate PC should be skipped");
        Ok(())
    }
}
