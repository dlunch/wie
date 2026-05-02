use alloc::{format, vec, vec::Vec};

use wie_util::{ByteRead, Result, WieError};

use crate::ArmCore;

mod hook;
mod parser;
mod patch;

use self::{
    hook::{Hook, PatternHook},
    patch::{PatchSpec, PatternPatchSpec},
};

/// Match the binary against the embedded patch table and install any matching
/// patches/hooks. The on-disk MD5 selects a hash-keyed entry first, falling
/// back to a hash-less generic entry. `scan_ranges` are the `(base, size)` byte
/// ranges searched for pattern matching (typically the guest `.text` region).
/// Returns the number of patches + hooks installed.
pub fn install_binary_patches(core: &mut ArmCore, data: &[u8], scan_ranges: &[(u32, u32)]) -> Result<usize> {
    let hash = md5::compute(data).0;
    let entries = parser::binary_patches();
    if let Some(entry) = entries.iter().find(|e| matches!(e.hash, Some(h) if h == hash)) {
        return install_entry(core, entry, scan_ranges, true);
    }
    if let Some(entry) = entries.iter().find(|e| e.hash.is_none()) {
        return install_entry(core, entry, scan_ranges, false);
    }
    Ok(0)
}

struct Entry {
    hash: Option<[u8; 16]>,
    name: alloc::string::String,
    hooks: Vec<Hook>,
    hook_patterns: Vec<PatternHook>,
    patches: Vec<PatchSpec>,
    patch_patterns: Vec<PatternPatchSpec>,
}

enum PatternToken {
    Literal(u8),
    AnyByte,
    Capture(CaptureName),
    /// Bit-level match for a byte that mixes a fixed opcode with a 3-bit
    /// register field (Thumb1 low-register encoding). `(byte & mask) == fixed`
    /// is the literal check; if `capture` is set, `(byte >> shift) & 0b111`
    /// is read into the named register slot, with cross-byte consistency
    /// enforced at match time.
    BitMatch {
        mask: u8,
        fixed: u8,
        capture: Option<(CaptureName, u8)>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CaptureName {
    Dst,
    Src,
    Len,
    ExitB,
    SrcReg,
    DstReg,
    CountReg,
}

#[derive(Debug, Clone)]
struct PatternMatch {
    addr: u32,
    dst: Option<u8>,
    src: Option<u8>,
    len: Option<u8>,
    src_reg: Option<u8>,
    dst_reg: Option<u8>,
    count_reg: Option<u8>,
    /// Address of the first `{exit_b}` byte — combined with `exit_b_bytes` to
    /// compute the branch target.
    exit_b_site: Option<u32>,
    exit_b_bytes: Option<[u8; 2]>,
}

fn install_entry(core: &mut ArmCore, entry: &Entry, scan_ranges: &[(u32, u32)], is_specific: bool) -> Result<usize> {
    let hooks = hook::resolve_hooks(core, entry, scan_ranges)?;
    let n_patches = patch::install_patches(core, entry, scan_ranges, &hooks)?;

    // A hash-keyed entry that produces zero installations is a strong signal
    // that the binary drifted from what the patch table targets. Generic
    // (hash-less) entries have no such guarantee. We trust the caller's
    // `is_specific` rather than `entry.hash.is_some()` so that a placeholder
    // all-zero hash in a fixture doesn't accidentally trigger this.
    if n_patches == 0 && hooks.is_empty() {
        if is_specific {
            return Err(WieError::FatalError(format!(
                "entry {}: hash matched but produced 0 patches/hooks (table likely out of date)",
                entry.name
            )));
        }
        tracing::warn!("entry {}: generic entry produced 0 patches/hooks", entry.name);
    }

    hook::apply_hooks(core, &entry.name, &hooks)?;
    Ok(n_patches + hooks.len())
}

/// Scans on 2-byte boundaries since all Thumb instructions are halfword-aligned.
fn scan_pattern(core: &mut ArmCore, tokens: &[PatternToken], scan_ranges: &[(u32, u32)]) -> Result<Vec<(u32, PatternMatch)>> {
    let mut results = Vec::new();
    let pat_len = tokens.len();
    if pat_len == 0 {
        return Ok(results);
    }

    for (base, size) in scan_ranges {
        let mut buf = vec![0u8; *size as usize];
        core.read_bytes(*base, &mut buf)?;

        let mut off = 0usize;
        while off + pat_len <= buf.len() {
            if let Some(mut pm) = try_match(tokens, &buf[off..off + pat_len]) {
                pm.addr = base + off as u32;
                if pm.exit_b_bytes.is_some() {
                    for (ti, t) in tokens.iter().enumerate() {
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
        src_reg: None,
        dst_reg: None,
        count_reg: None,
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
            PatternToken::BitMatch { mask, fixed, capture } => {
                if b & mask != *fixed {
                    return None;
                }
                if let Some((name, shift)) = capture {
                    let value = (b >> shift) & 0b111;
                    let slot = match name {
                        CaptureName::SrcReg => &mut m.src_reg,
                        CaptureName::DstReg => &mut m.dst_reg,
                        CaptureName::CountReg => &mut m.count_reg,
                        _ => return None,
                    };
                    if let Some(prev) = *slot
                        && prev != value
                    {
                        return None;
                    }
                    *slot = Some(value);
                }
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
                CaptureName::SrcReg | CaptureName::DstReg | CaptureName::CountReg => {
                    // Register captures only land in `BitMatch`, not whole-byte `Capture`.
                    return None;
                }
            },
        }
    }
    Some(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_toml_parses() {
        let entries = parser::binary_patches();
        assert!(!entries.is_empty(), "embedded TOML produced no entries");
        for entry in &entries {
            assert!(!entry.name.is_empty(), "entry missing name");
        }
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
    }

    #[test]
    fn pattern_bit_match_captures_register_at_correct_shift() {
        // `0b00sss011` matches `(src << 3) | 0b011`. Cross-byte consistency
        // forces the same `s` capture to agree across positions.
        let tokens = [
            PatternToken::BitMatch {
                mask: 0b1100_0111,
                fixed: 0b0000_0011,
                capture: Some((CaptureName::SrcReg, 3)),
            },
            PatternToken::BitMatch {
                mask: 0b1111_1000,
                fixed: 0b0011_0000,
                capture: Some((CaptureName::SrcReg, 0)),
            },
        ];
        // 0x0B = 0b00001011 → src bits 5..3 = 001 → R1
        // 0x31 = 0b00110001 → src bits 2..0 = 001 → R1 (consistent)
        let m = try_match(&tokens, &[0x0b, 0x31]).expect("match");
        assert_eq!(m.src_reg, Some(1));

        // Inconsistent capture must fail: first byte says R1, second says R2.
        assert!(try_match(&tokens, &[0x0b, 0x32]).is_none());
        // First byte's literal bits don't match (low nibble != 0b011).
        assert!(try_match(&tokens, &[0x0a, 0x31]).is_none());
    }

    #[test]
    fn pattern_scan_rejects_exit_b_capture_when_bytes_are_not_thumb_b() {
        let tokens = [PatternToken::Capture(CaptureName::ExitB), PatternToken::Capture(CaptureName::ExitB)];
        // `0c 3a` is `SUBS R2, #0xC` — high byte 0x3a is not in 0xE0-0xE7.
        assert!(try_match(&tokens, &[0x0c, 0x3a]).is_none());
        // `e8 e7` is a valid back-branch (high byte 0xE7).
        assert!(try_match(&tokens, &[0xe8, 0xe7]).is_some());
    }

    mod install_entry_tests {
        use alloc::{vec, vec::Vec};

        use wie_util::ByteWrite;

        use super::*;
        use crate::binary_patches::{
            hook::{Hook, HookKind},
            patch::{PatchSpec, PatternPatchSpec},
        };

        fn empty_entry(name: &str, hash: Option<[u8; 16]>) -> Entry {
            Entry {
                hash,
                name: name.into(),
                hooks: Vec::new(),
                hook_patterns: Vec::new(),
                patches: Vec::new(),
                patch_patterns: Vec::new(),
            }
        }

        #[test]
        fn install_entry_returns_patch_plus_hook_count() -> Result<()> {
            let mut core = ArmCore::new(false, None)?;
            core.map(0x2000, 0x100)?;
            core.write_bytes(0x2000, &[0xaa, 0xbb])?;

            let mut entry = empty_entry("count", Some([0u8; 16]));
            entry.patches.push(PatchSpec {
                pc: 0x2000,
                bytes: vec![0x11, 0x22],
                expect: Some(vec![0xaa, 0xbb]),
            });
            entry.hooks.push(Hook {
                pc: 0x2010 | 1,
                kind: HookKind::Memcpy,
            });

            let n = install_entry(&mut core, &entry, &[], true)?;
            assert_eq!(n, 2);
            Ok(())
        }

        #[test]
        fn install_entry_specific_entry_with_zero_results_is_fatal() -> Result<()> {
            let mut core = ArmCore::new(false, None)?;
            // Map a region with bytes that won't match the patch pattern.
            core.map(0x2000, 0x40)?;

            let mut entry = empty_entry("specific-empty", Some([1u8; 16]));
            entry.patch_patterns.push(PatternPatchSpec {
                tokens: vec![PatternToken::Literal(0xaa), PatternToken::Literal(0xbb)],
                bytes: vec![0x11, 0x22],
                expect: None,
                offset: 0,
            });

            let err = install_entry(&mut core, &entry, &[(0x2000, 0x40)], true).unwrap_err();
            assert!(format!("{err}").contains("hash matched but produced 0"));
            Ok(())
        }

        #[test]
        fn install_entry_generic_entry_with_zero_results_is_ok() -> Result<()> {
            let mut core = ArmCore::new(false, None)?;
            core.map(0x2000, 0x40)?;

            let mut entry = empty_entry("generic-empty", None);
            entry.patch_patterns.push(PatternPatchSpec {
                tokens: vec![PatternToken::Literal(0xaa), PatternToken::Literal(0xbb)],
                bytes: vec![0x11, 0x22],
                expect: None,
                offset: 0,
            });

            let n = install_entry(&mut core, &entry, &[(0x2000, 0x40)], false)?;
            assert_eq!(n, 0);
            Ok(())
        }

        #[test]
        fn install_entry_patch_pattern_overlapping_hook_pattern_is_fatal() -> Result<()> {
            // Hook pattern matches at 0x3000 (Thumb hook PC = 0x3001), patch
            // pattern matches at 0x3000 too — the SVC region [0x3000, 0x3002)
            // overlaps the patch [0x3000, 0x3002). Must fatal before any write.
            let mut core = ArmCore::new(false, None)?;
            core.map(0x3000, 0x40)?;
            core.write_bytes(0x3000, &[0x70, 0xb5, 0x00, 0x00])?;

            let mut entry = empty_entry("pat-overlap", None);
            entry.hook_patterns.push(crate::binary_patches::hook::PatternHook {
                tokens: vec![PatternToken::Literal(0x70), PatternToken::Literal(0xb5)],
                kind_template: crate::binary_patches::hook::PatternHookKind::Memcpy,
            });
            entry.patch_patterns.push(PatternPatchSpec {
                tokens: vec![PatternToken::Literal(0x70), PatternToken::Literal(0xb5)],
                bytes: vec![0x00, 0x00],
                expect: None,
                offset: 0,
            });

            let err = install_entry(&mut core, &entry, &[(0x3000, 0x40)], false).unwrap_err();
            assert!(format!("{err}").contains("overlap"), "{err}");
            // Memory unchanged.
            let mut buf = [0u8; 2];
            core.read_bytes(0x3000, &mut buf)?;
            assert_eq!(buf, [0x70, 0xb5]);
            Ok(())
        }

        #[test]
        fn install_entry_applies_patch_before_hook() -> Result<()> {
            // Patch at [0x4000, 0x4002), hook at 0x4003 (Thumb, SVC at
            // [0x4002, 0x4004)). Adjacent regions, no overlap. Both writes
            // must end up in memory.
            let mut core = ArmCore::new(false, None)?;
            core.map(0x4000, 0x40)?;
            core.write_bytes(0x4000, &[0xaa, 0xbb, 0xcc, 0xdd])?;

            let mut entry = empty_entry("ordered", Some([0u8; 16]));
            entry.patches.push(PatchSpec {
                pc: 0x4000,
                bytes: vec![0x11, 0x22],
                expect: Some(vec![0xaa, 0xbb]),
            });
            entry.hooks.push(Hook {
                pc: 0x4003,
                kind: HookKind::Memcpy,
            });

            let n = install_entry(&mut core, &entry, &[], true)?;
            assert_eq!(n, 2);

            let mut buf = [0u8; 4];
            core.read_bytes(0x4000, &mut buf)?;
            assert_eq!(&buf[0..2], &[0x11, 0x22], "patch applied");
            assert_eq!(buf[3], 0xdf, "hook SVC high byte");
            assert_eq!(buf[2] as u32, 0x80, "hook SVC low byte = SVC #0x80");
            Ok(())
        }

        #[test]
        fn install_entry_pattern_patch_expect_mismatch_is_fatal() -> Result<()> {
            let mut core = ArmCore::new(false, None)?;
            core.map(0x5000, 0x40)?;
            core.write_bytes(0x5000, &[0x10, 0x20, 0x30, 0x40])?;

            let mut entry = empty_entry("pp-mm", None);
            entry.patch_patterns.push(PatternPatchSpec {
                tokens: vec![
                    PatternToken::Literal(0x10),
                    PatternToken::Literal(0x20),
                    PatternToken::Literal(0x30),
                    PatternToken::Literal(0x40),
                ],
                bytes: vec![0xaa, 0xbb],
                expect: Some(vec![0x99, 0x99]), // doesn't match what's there
                offset: 1,
            });

            let err = install_entry(&mut core, &entry, &[(0x5000, 0x40)], false).unwrap_err();
            assert!(format!("{err}").contains("expected"), "{err}");
            // Memory unchanged.
            let mut buf = [0u8; 4];
            core.read_bytes(0x5000, &mut buf)?;
            assert_eq!(buf, [0x10, 0x20, 0x30, 0x40]);
            Ok(())
        }
    }
}
