use alloc::{collections::BTreeMap, format, string::ToString, sync::Arc, vec::Vec};

use wie_util::{ByteWrite, Result, WieError, read_generic};

use super::{Entry, PatternToken, scan_pattern};
use crate::{ArmCore, engine::ArmRegister, function::JumpTo, stdlib};

const BINARY_PATCH_SVC: u32 = 0x80;

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
    /// Replaces an inline byte-copy loop whose src/dst/count live in registers
    /// instead of on the stack. Bytes copied = `read(count) + count_offset`.
    /// After the copy, `src`/`dst` are advanced by that count and `count` is
    /// rewritten so it equals what the loop would have left there (i.e.,
    /// `original - bytes`), then the dispatcher jumps to `exit_pc`.
    RegInlineCopy(RegInlineCopy),
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

#[derive(Debug, Clone, Copy)]
pub struct RegInlineCopy {
    src: ArmRegister,
    dst: ArmRegister,
    count: ArmRegister,
    count_offset: i32,
    exit_pc: u32,
}

/// Scanned across the install-time memory range; each match becomes a `Hook`.
pub struct PatternHook {
    pub tokens: Vec<PatternToken>,
    pub kind_template: PatternHookKind,
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
    /// Pattern-matched register-resident copy loop. The `src`/`dst`/`count`
    /// registers are read from the pattern's bit-level captures and `exit_pc`
    /// is derived at install time from `match_addr + pattern.len()` (Thumb bit
    /// set).
    RegInlineCopy {
        count_offset: i32,
    },
}

/// Expand static + pattern hooks into a single `Vec<Hook>` whose PCs are final
/// and Thumb-valid. All pattern matching happens here; downstream consumers
/// (overlap check, `apply_hooks`) only see PC + kind, never raw tokens.
///
/// Static hook PCs come from TOML, so we validate the Thumb bit up front —
/// `install_entry` runs `apply_patches` between resolve and apply, and a
/// fatal-after-write would leave guest memory partially modified.
pub fn resolve_hooks(core: &mut ArmCore, entry: &Entry, scan_ranges: &[(u32, u32)]) -> Result<Vec<Hook>> {
    for hook in &entry.hooks {
        if hook.pc & 1 == 0 {
            return Err(WieError::FatalError(format!(
                "entry {}: hook PC {:#x} targets ARM mode; only Thumb (LSB=1) is supported",
                entry.name, hook.pc
            )));
        }
    }
    let mut installed: Vec<Hook> = entry.hooks.clone();

    for pattern in &entry.hook_patterns {
        let matches = scan_pattern(core, &pattern.tokens, scan_ranges)?;
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
                PatternHookKind::RegInlineCopy { count_offset } => {
                    let src = arm_register_from_index(
                        pm.src_reg
                            .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing src register capture")))?,
                    );
                    let dst = arm_register_from_index(
                        pm.dst_reg
                            .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing dst register capture")))?,
                    );
                    let count = arm_register_from_index(
                        pm.count_reg
                            .ok_or_else(|| WieError::FatalError(format!("pattern match at {match_addr:#x} missing count register capture")))?,
                    );
                    let exit_pc = match_addr.wrapping_add(pattern.tokens.len() as u32) | 1;
                    HookKind::RegInlineCopy(RegInlineCopy {
                        src,
                        dst,
                        count,
                        count_offset: *count_offset,
                        exit_pc,
                    })
                }
            };
            let pc = match_addr | 1;
            if installed.iter().any(|h| h.pc == pc) {
                tracing::warn!("Hook at {pc:#x} already registered; skipping duplicate match");
                continue;
            }
            installed.push(Hook { pc, kind });
        }
    }

    Ok(installed)
}

/// Patch the SVC instruction at every hook PC and register the dispatcher.
/// `hooks` must already be the fully expanded list from `resolve_hooks`. The
/// dispatcher is registered even when `hooks` is empty so that any later SVC
/// #0x80 (e.g., from guest code that happens to encode the same bytes) routes
/// to a single, named diagnostic instead of falling out of the engine.
pub fn apply_hooks(core: &mut ArmCore, entry_name: &str, hooks: &[Hook]) -> Result<()> {
    let mut registry = BTreeMap::new();
    for hook in hooks {
        debug_assert!(hook.pc & 1 == 1, "resolve_hooks must reject ARM-mode PCs");
        registry.insert(hook.pc, hook.kind);
        let patch_addr = hook.pc & !1;
        let instruction: u16 = 0xdf00 | (BINARY_PATCH_SVC as u16 & 0xff);
        core.write_bytes(patch_addr, &instruction.to_le_bytes())?;
        tracing::info!("Hook installed at {:#x}: {:?}", hook.pc, hook.kind);
    }
    if !hooks.is_empty() {
        tracing::info!("Installed {} hooks for {}", hooks.len(), entry_name);
    }
    core.register_svc_handler(BINARY_PATCH_SVC, handle_binary_patch_svc, &Arc::new(registry))?;
    Ok(())
}

/// Negate the unsigned `SUBS Rn, #imm8` immediate: the captured byte is the
/// distance below R7, so the resulting offset is `-imm8`.
fn capture_to_offset(byte: u8) -> i32 {
    -(byte as i32)
}

fn arm_register_from_index(index: u8) -> ArmRegister {
    match index {
        0 => ArmRegister::R0,
        1 => ArmRegister::R1,
        2 => ArmRegister::R2,
        3 => ArmRegister::R3,
        4 => ArmRegister::R4,
        5 => ArmRegister::R5,
        6 => ArmRegister::R6,
        7 => ArmRegister::R7,
        _ => unreachable!("BitMatch only captures 3 bits, value must be 0..=7"),
    }
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

type Registry = Arc<BTreeMap<u32, HookKind>>;

async fn handle_binary_patch_svc(core: &mut ArmCore, registry: &mut Registry) -> Result<JumpTo> {
    let (pc, lr) = core.read_pc_lr()?;
    // PC on entry is the address right after the patched 2-byte SVC. Drop any
    // Thumb bit, step back over the SVC, then re-set the Thumb bit because
    // hook PCs are stored that way.
    let hook_pc = (pc.wrapping_sub(2)) | 1;
    let kind = registry
        .get(&hook_pc)
        .copied()
        .ok_or_else(|| WieError::FatalError(format!("binary-patch hook fired at unregistered PC {hook_pc:#x}")))?;

    match kind {
        HookKind::Memcpy => {
            let (dst, src, len) = {
                let inner = core.inner.lock();
                (
                    inner.engine.reg_read(ArmRegister::R0),
                    inner.engine.reg_read(ArmRegister::R1),
                    inner.engine.reg_read(ArmRegister::R2),
                )
            };
            tracing::trace!("hook memcpy(ptr_dst={dst:#x}, ptr_src={src:#x}, len={len:#x})");
            stdlib::memcpy(core, &mut (), dst, src, len).await?;
            Ok(JumpTo(lr))
        }
        HookKind::Memset => {
            let (dst, val, len) = {
                let inner = core.inner.lock();
                (
                    inner.engine.reg_read(ArmRegister::R0),
                    inner.engine.reg_read(ArmRegister::R1),
                    inner.engine.reg_read(ArmRegister::R2),
                )
            };
            tracing::trace!("hook memset(ptr_dst={dst:#x}, val={:#x}, len={len:#x})", val as u8);
            stdlib::memset(core, &mut (), dst, val, len).await?;
            Ok(JumpTo(lr))
        }
        HookKind::Strcpy => {
            let (dst, src) = {
                let inner = core.inner.lock();
                (inner.engine.reg_read(ArmRegister::R0), inner.engine.reg_read(ArmRegister::R1))
            };
            tracing::trace!("hook strcpy(ptr_dst={dst:#x}, ptr_src={src:#x})");
            stdlib::strcpy(core, &mut (), dst, src).await?;
            Ok(JumpTo(lr))
        }
        HookKind::Strlen => {
            let s = core.inner.lock().engine.reg_read(ArmRegister::R0);
            let len = stdlib::strlen(core, &mut (), s).await?;
            tracing::trace!("hook strlen(ptr_str={s:#x}) -> {len:#x}");
            core.inner.lock().engine.reg_write(ArmRegister::R0, len);
            Ok(JumpTo(lr))
        }
        HookKind::InlineCopy(spec) => {
            let r7 = core.inner.lock().engine.reg_read(ArmRegister::R7);
            let dst_slot = r7.wrapping_add(spec.dst_offset as u32);
            let src_slot = r7.wrapping_add(spec.src_offset as u32);
            let len_slot = r7.wrapping_add(spec.len_offset as u32);
            let dst: u32 = read_generic(core, dst_slot)?;
            let src: u32 = read_generic(core, src_slot)?;
            let len: u32 = read_generic(core, len_slot)?;
            tracing::trace!(
                "hook inline_copy(ptr_dst={dst:#x}, ptr_src={src:#x}, len={len:#x}, exit={:#x})",
                spec.exit_pc
            );
            stdlib::memcpy(core, &mut (), dst, src, len).await?;
            if spec.spill_back {
                core.write_bytes(dst_slot, &dst.wrapping_add(len).to_le_bytes())?;
                core.write_bytes(src_slot, &src.wrapping_add(len).to_le_bytes())?;
                core.write_bytes(len_slot, &0u32.to_le_bytes())?;
            }
            Ok(JumpTo(spec.exit_pc))
        }
        HookKind::RegInlineCopy(spec) => {
            let (src, dst, count_initial) = {
                let inner = core.inner.lock();
                (
                    inner.engine.reg_read(spec.src),
                    inner.engine.reg_read(spec.dst),
                    inner.engine.reg_read(spec.count),
                )
            };
            let count = count_initial.wrapping_add(spec.count_offset as u32);
            tracing::trace!(
                "hook reg_inline_copy(src={src:#x}, dst={dst:#x}, count={count:#x}, exit={:#x})",
                spec.exit_pc
            );
            stdlib::memcpy(core, &mut (), dst, src, count).await?;
            let mut inner = core.inner.lock();
            inner.engine.reg_write(spec.src, src.wrapping_add(count));
            inner.engine.reg_write(spec.dst, dst.wrapping_add(count));
            inner.engine.reg_write(spec.count, count_initial.wrapping_sub(count));
            Ok(JumpTo(spec.exit_pc))
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use wie_util::ByteRead;

    use super::*;
    use crate::function::{RegisteredFunction, RegisteredFunctionHolder};

    fn registry_with(pc: u32, kind: HookKind) -> Registry {
        let mut map = BTreeMap::new();
        map.insert(pc, kind);
        Arc::new(map)
    }

    /// Set PC to where it would be on entry to the SVC handler for a hook at
    /// `hook_pc` (svc_addr + 2, i.e. just past the patched 2-byte SVC).
    fn set_post_svc_pc(core: &mut ArmCore, hook_pc: u32) {
        let mut inner = core.inner.lock();
        inner.engine.reg_write(ArmRegister::PC, (hook_pc & !1).wrapping_add(2));
    }

    fn entry_with_static(name: &str, hooks: Vec<Hook>) -> Entry {
        Entry {
            hash: Some([0u8; 16]),
            name: name.into(),
            hooks,
            hook_patterns: vec![],
            patches: vec![],
            patch_patterns: vec![],
        }
    }

    #[test]
    fn resolve_hooks_rejects_arm_mode_pc() -> Result<()> {
        let entry = entry_with_static(
            "arm-mode",
            vec![Hook {
                pc: 0x2000, // LSB=0 => ARM mode
                kind: HookKind::Memcpy,
            }],
        );
        let mut core = ArmCore::new(false, None)?;
        core.map(0x2000, 0x1000)?;

        let err = resolve_hooks(&mut core, &entry, &[]).unwrap_err();
        let msg = alloc::format!("{err}");
        assert!(msg.contains("ARM mode"), "unexpected error: {msg}");
        Ok(())
    }

    #[test]
    fn apply_hooks_writes_thumb_svc_instruction() -> Result<()> {
        let entry = entry_with_static(
            "patch",
            vec![Hook {
                pc: 0x2001, // Thumb
                kind: HookKind::Memcpy,
            }],
        );
        let mut core = ArmCore::new(false, None)?;
        core.map(0x2000, 0x1000)?;
        core.write_bytes(0x2000, &[0xaa, 0xbb])?;

        let hooks = resolve_hooks(&mut core, &entry, &[])?;
        apply_hooks(&mut core, &entry.name, &hooks)?;

        let mut buf = [0u8; 2];
        core.read_bytes(0x2000, &mut buf)?;
        assert_eq!(buf, [BINARY_PATCH_SVC as u8, 0xdf]);
        Ok(())
    }

    #[futures_test::test]
    async fn memcpy_dispatch_copies_bytes_and_returns_via_lr() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x1000)?;

        let src = 0x10000u32;
        let dst = 0x10400u32;
        let data = [1u8, 2, 3, 4, 5, 6, 7, 8];
        core.write_bytes(src, &data)?;

        let hook_pc = 0x10001u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, src);
            inner.engine.reg_write(ArmRegister::R2, data.len() as u32);
            inner.engine.reg_write(ArmRegister::LR, 0xdead_beef);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let registry = registry_with(hook_pc, HookKind::Memcpy);
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

        let mut out = [0u8; 8];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, data);
        assert_eq!(core.inner.lock().engine.reg_read(ArmRegister::PC), 0xdead_beef & !1);
        Ok(())
    }

    #[futures_test::test]
    async fn memset_dispatch_fills_bytes() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x1000)?;

        let dst = 0x10000u32;
        let len = 16u32;
        let hook_pc = 0x10401u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, 0xab);
            inner.engine.reg_write(ArmRegister::R2, len);
            inner.engine.reg_write(ArmRegister::LR, 0x3000);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let registry = registry_with(hook_pc, HookKind::Memset);
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

        let mut out = [0u8; 16];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, [0xab; 16]);
        Ok(())
    }

    #[futures_test::test]
    async fn strcpy_dispatch_copies_null_terminated_string_and_returns_via_lr() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x1000)?;

        let src = 0x10000u32;
        let dst = 0x10400u32;
        let s = b"hello, world!\0";
        core.write_bytes(src, s)?;
        core.write_bytes(dst, &[0xff; 32])?;

        let hook_pc = 0x10801u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, dst);
            inner.engine.reg_write(ArmRegister::R1, src);
            inner.engine.reg_write(ArmRegister::LR, 0xcafe_babe);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let registry = registry_with(hook_pc, HookKind::Strcpy);
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

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
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x1000)?;

        let str_ptr = 0x10100u32;
        let s = b"abcdef\0";
        core.write_bytes(str_ptr, s)?;

        let hook_pc = 0x10c01u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R0, str_ptr);
            inner.engine.reg_write(ArmRegister::LR, 0x1234_5678);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let registry = registry_with(hook_pc, HookKind::Strlen);
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::R0), 6);
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), 0x1234_5678 & !1);
        Ok(())
    }

    #[futures_test::test]
    async fn inline_copy_dispatch_reads_frame_copies_and_jumps_to_exit() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x10000, 0x2000)?;

        let src = 0x10000u32;
        let dst = 0x10800u32;
        let payload = [0xaa, 0xbb, 0xcc, 0xdd];
        core.write_bytes(src, &payload)?;

        let frame = 0x11000u32;
        core.write_bytes(frame, &dst.to_le_bytes())?;
        core.write_bytes(frame + 4, &src.to_le_bytes())?;
        core.write_bytes(frame + 8, &(payload.len() as u32).to_le_bytes())?;

        let hook_pc = 0x10201u32;
        {
            let mut inner = core.inner.lock();
            inner.engine.reg_write(ArmRegister::R7, frame);
        }
        set_post_svc_pc(&mut core, hook_pc);

        let spec = InlineCopy {
            dst_offset: 0,
            src_offset: 4,
            len_offset: 8,
            exit_pc: 0x10401,
            spill_back: true,
        };
        let registry = registry_with(hook_pc, HookKind::InlineCopy(spec));
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry).call(&mut core).await?;

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

        assert_eq!(core.inner.lock().engine.reg_read(ArmRegister::PC), 0x10400);
        Ok(())
    }

    #[futures_test::test]
    async fn install_then_execute_hits_dispatcher_end_to_end() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x20000, 0x2000)?;
        core.map(0x30000, 0x1000)?;

        let src = 0x30000u32;
        let dst = 0x30200u32;
        let payload = [9u8, 8, 7, 6];
        core.write_bytes(src, &payload)?;

        let hook_pc = 0x20001u32;
        let entry = entry_with_static(
            "e2e",
            vec![Hook {
                pc: hook_pc,
                kind: HookKind::Memcpy,
            }],
        );
        let hooks = resolve_hooks(&mut core, &entry, &[])?;
        apply_hooks(&mut core, &entry.name, &hooks)?;

        let mut opcode = [0u8; 2];
        core.read_bytes(0x20000, &mut opcode)?;
        assert_eq!(opcode[1], 0xdf);
        assert_eq!(opcode[0] as u32, BINARY_PATCH_SVC);

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
        assert_eq!(category, BINARY_PATCH_SVC);

        let registry = registry_with(hook_pc, HookKind::Memcpy);
        let mut core_clone = core.clone();
        RegisteredFunctionHolder::new(handle_binary_patch_svc, &registry)
            .call(&mut core_clone)
            .await?;

        let mut out = [0u8; 4];
        core.read_bytes(dst, &mut out)?;
        assert_eq!(out, payload);

        let inner = core.inner.lock();
        assert_eq!(inner.engine.reg_read(ArmRegister::PC), return_addr & !1);
        Ok(())
    }

    #[test]
    fn pattern_scan_matches_single_hit() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x50000, 0x200)?;

        let pat_bytes = [0xaa, 0xbb, 0xcc, 0xdd];
        let match_addr = 0x50020u32;
        core.write_bytes(match_addr, &pat_bytes)?;

        let entry = Entry {
            hash: None,
            name: "scan-single".into(),
            hooks: vec![],
            hook_patterns: vec![PatternHook {
                tokens: vec![
                    PatternToken::Literal(0xaa),
                    PatternToken::Literal(0xbb),
                    PatternToken::Literal(0xcc),
                    PatternToken::Literal(0xdd),
                ],
                kind_template: PatternHookKind::Memcpy,
            }],
            patches: vec![],
            patch_patterns: vec![],
        };

        let hooks = resolve_hooks(&mut core, &entry, &[(0x50000, 0x200)])?;
        apply_hooks(&mut core, &entry.name, &hooks)?;

        let mut out = [0u8; 2];
        core.read_bytes(match_addr, &mut out)?;
        assert_eq!(out[1], 0xdf);
        assert_eq!(out[0] as u32, BINARY_PATCH_SVC);
        Ok(())
    }

    #[test]
    fn pattern_scan_capture_to_offset_negates_imm8() {
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
    fn pattern_duplicate_pc_warns_once_and_skips() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        core.map(0x60000, 0x100)?;
        core.write_bytes(0x60010, &[0x11, 0x22, 0x33, 0x44])?;

        let entry = Entry {
            hash: None,
            name: "dup".into(),
            hooks: vec![],
            hook_patterns: vec![
                PatternHook {
                    tokens: vec![PatternToken::Literal(0x11), PatternToken::Literal(0x22)],
                    kind_template: PatternHookKind::Memcpy,
                },
                PatternHook {
                    tokens: vec![PatternToken::Literal(0x11), PatternToken::Literal(0x22)],
                    kind_template: PatternHookKind::Memcpy,
                },
            ],
            patches: vec![],
            patch_patterns: vec![],
        };
        let hooks = resolve_hooks(&mut core, &entry, &[(0x60000, 0x100)])?;
        assert_eq!(hooks.len(), 1, "duplicate PC should be skipped");
        Ok(())
    }
}
