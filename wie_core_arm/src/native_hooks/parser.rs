use alloc::{string::String, vec::Vec};

use serde::Deserialize;

use super::{CaptureName, Entry, Hook, HookKind, InlineCopy, PatternHook, PatternHookKind, PatternToken};

const NATIVE_HOOKS_TOML: &str = include_str!("../../../data/native_hooks.toml");

pub fn native_hooks() -> Vec<Entry> {
    let doc: RawDoc = toml::from_str(NATIVE_HOOKS_TOML).expect("parse data/native_hooks.toml");
    doc.entry.into_iter().map(RawEntry::into_entry).collect()
}

#[derive(Deserialize)]
struct RawDoc {
    entry: Vec<RawEntry>,
}

#[derive(Deserialize)]
struct RawEntry {
    hash: Option<String>,
    name: String,
    hook: Vec<RawHook>,
}

#[derive(Deserialize)]
struct RawHook {
    kind: KindTag,
    pc: Option<u32>,
    pattern: Option<String>,
    dst_offset: Option<i32>,
    src_offset: Option<i32>,
    len_offset: Option<i32>,
    exit_pc: Option<u32>,
    spill_back: Option<bool>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum KindTag {
    Memcpy,
    Memset,
    Strcpy,
    Strlen,
    InlineCopy,
}

impl RawEntry {
    fn into_entry(self) -> Entry {
        let hash = self.hash.as_deref().map(parse_hash);
        let name = self.name;
        let mut hooks = Vec::new();
        let mut patterns = Vec::new();
        for raw in self.hook {
            match (raw.pc, raw.pattern.as_deref()) {
                (Some(pc), None) => hooks.push(Hook {
                    pc,
                    kind: pc_kind(&raw, &name),
                }),
                (None, Some(pat)) => {
                    let tokens = parse_pattern(pat, &name);
                    let kind_template = pattern_template(&raw, &tokens, &name);
                    patterns.push(PatternHook { tokens, kind_template });
                }
                (Some(_), Some(_)) => panic!("entry {name}: hook cannot specify both `pc` and `pattern`"),
                (None, None) => panic!("entry {name}: hook must specify either `pc` or `pattern`"),
            }
        }
        if hash.is_none() && !hooks.is_empty() {
            panic!("entry {name}: hash is required when pc-based hooks are present (a pc only makes sense for a specific binary)");
        }
        Entry { hash, name, hooks, patterns }
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
    }
}

fn pattern_template(raw: &RawHook, tokens: &[PatternToken], entry_name: &str) -> PatternHookKind {
    match raw.kind {
        KindTag::Memcpy => PatternHookKind::Memcpy,
        KindTag::Memset => PatternHookKind::Memset,
        KindTag::Strcpy => PatternHookKind::Strcpy,
        KindTag::Strlen => PatternHookKind::Strlen,
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
