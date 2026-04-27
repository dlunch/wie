use alloc::{format, string::ToString, vec, vec::Vec};

use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic};

use crate::{ArmCore, engine::ArmRegister, function::JumpTo, stdlib};

/// Entry for a single game binary. When `hash` is set it must equal the MD5
/// of the on-disk binary payload (pre-relocation/self-patching) for the entry
/// to install. Entries with `hash = None` install unconditionally — only
/// valid when every hook is pattern-based, since pattern matches are
/// self-validating.
pub struct Entry {
    pub hash: Option<[u8; 16]>,
    pub name: &'static str,
    pub hooks: &'static [Hook],
    pub patterns: &'static [PatternHook],
}

#[derive(Debug, Clone, Copy)]
pub struct Hook {
    /// Target PC. LSB=1 denotes Thumb mode (required for now; ARM SVC is not
    /// supported by the underlying engine).
    pub pc: u32,
    pub kind: HookKind,
}

#[derive(Debug, Clone, Copy)]
pub enum HookKind {
    /// ARM ABI function entry: dst=r0, src=r1, len=r2. After the copy, return
    /// to the caller via LR.
    Memcpy,
    /// ARM ABI function entry: dst=r0, val=r1 (low byte), len=r2. Return via LR.
    Memset,
    /// ARM ABI function entry: dst=r0, src=r1. Copy NUL-terminated string
    /// (including the NUL). Return original `dst` via r0, then return via LR.
    Strcpy,
    /// ARM ABI function entry: str=r0. Compute length up to (not including)
    /// the NUL terminator. Return length in r0, then return via LR.
    Strlen,
    /// Inline byte-copy loop with `dst`/`src`/`len` on the current stack frame
    /// relative to R7 (typical Thumb frame pointer). The loop must use a
    /// down-counter (`len` decremented to 0 to exit). After the copy the
    /// dispatcher jumps to `exit_pc` to resume execution past the loop.
    InlineCopy {
        dst_offset: i32,
        src_offset: i32,
        len_offset: i32,
        exit_pc: u32,
        /// If true, write back dst+len, src+len, len=0 into the spill slots.
        /// Needed when the compiler emits code after the loop that re-reads
        /// those variables.
        spill_back: bool,
    },
}

/// Pattern-based hook: scanned across a memory range at install time to
/// discover one or more concrete sites, each materialized as a `Hook`.
pub struct PatternHook {
    pub tokens: &'static [PatternToken],
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
        /// `None` => runtime captures the value from `{dst}` / `{src}` / `{len}`
        /// using the Thumb1 `SUBS Rn, #imm8` encoding convention:
        /// the captured byte `b` is reinterpreted as `-(b as i8) as i32`.
        /// `Some(v)` => fixed offset from TOML (used when pattern omits the capture).
        dst_offset: Option<i32>,
        src_offset: Option<i32>,
        len_offset: Option<i32>,
        /// `None` => `{exit_b}` capture fills this at runtime.
        exit_pc: Option<u32>,
        spill_back: bool,
    },
}

pub const NATIVE_HOOKS: &[Entry] = include!(concat!(env!("OUT_DIR"), "/native_hooks_data.rs"));

/// Reserved SVC category range for native hooks. `NATIVE_HOOK_CATEGORY_BASE +
/// hook_id` is registered per hook. 0x80..0xFF is above every category used
/// by KTF/LGT/SKT (<=18), yet still fits in the 8-bit Thumb SVC immediate.
pub const NATIVE_HOOK_CATEGORY_BASE: u32 = 0x80;
pub const NATIVE_HOOK_MAX: usize = 0x80;

/// Result of matching a `PatternHook` at a concrete address.
#[derive(Debug, Clone)]
struct PatternMatch {
    addr: u32,
    dst: Option<u8>,
    src: Option<u8>,
    len: Option<u8>,
    /// Address of the first `{exit_b}` byte (needed to compute the branch target).
    exit_b_site: Option<u32>,
    /// Two-byte Thumb B encoding captured from the pattern.
    exit_b_bytes: Option<[u8; 2]>,
}

/// Per-hook context passed to `handle_inline_copy`. Carries the stack-frame
/// offsets and the exit PC the dispatcher must resume at after the copy.
#[derive(Clone)]
struct InlineCopyCtx {
    dst_offset: i32,
    src_offset: i32,
    len_offset: i32,
    exit_pc: u32,
    spill_back: bool,
}

pub fn md5(data: &[u8]) -> [u8; 16] {
    md5::compute(data).0
}

/// Install the native hooks described by `entry`. Patches each hook site
/// with an `svc` instruction and registers a dispatcher per hook id.
///
/// `scan_ranges` are the (base, size) byte ranges searched for pattern-based
/// hooks (typically the guest `.text` region).
///
/// Returns the number of hooks actually installed (duplicate-PC matches are
/// skipped). Errors if any hook PC targets ARM mode (LSB=0); the current
/// engine only services Thumb SVC exceptions.
pub fn install(core: &mut ArmCore, entry: &'static Entry, scan_ranges: &[(u32, u32)]) -> Result<usize> {
    let mut installed: Vec<Hook> = entry.hooks.to_vec();

    for pattern in entry.patterns {
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
                    HookKind::InlineCopy {
                        dst_offset: dst,
                        src_offset: src,
                        len_offset: len,
                        exit_pc: exit,
                        spill_back: *spill_back,
                    }
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
            HookKind::InlineCopy {
                dst_offset,
                src_offset,
                len_offset,
                exit_pc,
                spill_back,
            } => {
                let ctx = InlineCopyCtx {
                    dst_offset,
                    src_offset,
                    len_offset,
                    exit_pc,
                    spill_back,
                };
                core.register_svc_handler(category, handle_inline_copy, &ctx)?;
            }
        }

        let patch_addr = hook.pc & !1;
        let instruction: u16 = 0xdf00 | (category as u16 & 0xff);
        core.write_bytes(patch_addr, &instruction.to_le_bytes())?;

        tracing::info!("Native hook installed at {:#x}: {:?}", hook.pc, hook.kind);
    }

    Ok(installed.len())
}

/// Scan the configured ranges for `pattern`. Matches are checked on 2-byte
/// (halfword) boundaries — all Thumb instructions are halfword-aligned.
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
            if let Some(mut pm) = try_match(pattern.tokens, &buf[off..off + pat_len]) {
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

/// Try to match `tokens` against `bytes` starting at logical offset 0.
/// Returns a `PatternMatch` with captures populated (addr/site computed by caller).
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

/// Convert a captured Thumb1 `SUBS Rn, #imm8` byte to the signed stack offset
/// used by `InlineCopy`. Typical frame pointers are below the slot, so the
/// compiler emits a SUBS to reach them — we negate the immediate.
fn capture_to_offset(byte: u8) -> i32 {
    -(byte as i8 as i32)
}

/// Decode a two-byte Thumb unconditional forward branch (`11100 imm11`).
/// `b_site` is the byte address of the instruction. Returns the target PC
/// with the Thumb bit set.
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

async fn handle_inline_copy(core: &mut ArmCore, ctx: &mut InlineCopyCtx) -> Result<JumpTo> {
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
    fn install_rejects_arm_mode_pc() -> Result<()> {
        static ENTRY: Entry = Entry {
            hash: Some([0u8; 16]),
            name: "arm-mode",
            hooks: &[Hook {
                pc: 0x2000, // LSB=0 => ARM mode
                kind: HookKind::Memcpy,
            }],
            patterns: &[],
        };
        let mut core = ArmCore::new(false)?;
        core.map(0x2000, 0x1000)?;

        let err = install(&mut core, &ENTRY, &[]).unwrap_err();
        let msg = alloc::format!("{err}");
        assert!(msg.contains("ARM mode"), "unexpected error: {msg}");
        Ok(())
    }

    #[test]
    fn install_patches_thumb_svc_instruction() -> Result<()> {
        static ENTRY: Entry = Entry {
            hash: Some([0u8; 16]),
            name: "patch",
            hooks: &[Hook {
                pc: 0x2001, // Thumb
                kind: HookKind::Memcpy,
            }],
            patterns: &[],
        };
        let mut core = ArmCore::new(false)?;
        core.map(0x2000, 0x1000)?;

        core.write_bytes(0x2000, &[0xaa, 0xbb])?;

        install(&mut core, &ENTRY, &[])?;

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

        let ctx = InlineCopyCtx {
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
        static ENTRY: Entry = Entry {
            hash: Some([0u8; 16]),
            name: "e2e",
            hooks: &[Hook {
                pc: 0x20001,
                kind: HookKind::Memcpy,
            }],
            patterns: &[],
        };
        install(&mut core, &ENTRY, &[])?;

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

        static ENTRY: Entry = Entry {
            hash: None,
            name: "scan-single",
            hooks: &[],
            patterns: &[PatternHook {
                tokens: &[
                    PatternToken::Literal(0xaa),
                    PatternToken::Literal(0xbb),
                    PatternToken::Literal(0xcc),
                    PatternToken::Literal(0xdd),
                ],
                kind_template: PatternHookKind::Memcpy,
            }],
        };

        install(&mut core, &ENTRY, &[(0x50000, 0x200)])?;

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

        static ENTRY: Entry = Entry {
            hash: None,
            name: "dup",
            hooks: &[],
            patterns: &[
                PatternHook {
                    tokens: &[PatternToken::Literal(0x11), PatternToken::Literal(0x22)],
                    kind_template: PatternHookKind::Memcpy,
                },
                PatternHook {
                    tokens: &[PatternToken::Literal(0x11), PatternToken::Literal(0x22)],
                    kind_template: PatternHookKind::Memcpy,
                },
            ],
        };
        let count = install(&mut core, &ENTRY, &[(0x60000, 0x100)])?;
        assert_eq!(count, 1, "duplicate PC should be skipped");
        Ok(())
    }
}
