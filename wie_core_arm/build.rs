use std::{env, fs, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct Document {
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

#[derive(Debug, Clone)]
enum Token {
    Literal(u8),
    AnyByte,
    Capture(CaptureName),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureName {
    Dst,
    Src,
    Len,
    ExitB,
}

impl CaptureName {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "dst" => Some(Self::Dst),
            "src" => Some(Self::Src),
            "len" => Some(Self::Len),
            "exit_b" => Some(Self::ExitB),
            _ => None,
        }
    }

    fn ident(self) -> &'static str {
        match self {
            Self::Dst => "Dst",
            Self::Src => "Src",
            Self::Len => "Len",
            Self::ExitB => "ExitB",
        }
    }
}

fn parse_pattern(pattern: &str, entry_name: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for raw in pattern.split_whitespace() {
        if raw == "??" {
            tokens.push(Token::AnyByte);
        } else if let Some(rest) = raw.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            let cap = CaptureName::parse(rest)
                .unwrap_or_else(|| panic!("entry {entry_name}: unknown capture name {{{rest}}} (allowed: dst, src, len, exit_b)"));
            tokens.push(Token::Capture(cap));
        } else if raw.len() == 2 && raw.chars().all(|c| c.is_ascii_hexdigit()) {
            let byte = u8::from_str_radix(raw, 16).unwrap();
            tokens.push(Token::Literal(byte));
        } else {
            panic!("entry {entry_name}: invalid pattern token `{raw}`");
        }
    }
    tokens
}

fn validate_exit_b(tokens: &[Token], entry_name: &str) {
    let mut pair_seen = false;
    let mut i = 0;
    while i < tokens.len() {
        if let Token::Capture(CaptureName::ExitB) = tokens[i] {
            let has_next = i + 1 < tokens.len() && matches!(tokens[i + 1], Token::Capture(CaptureName::ExitB));
            if !has_next {
                panic!("entry {entry_name}: {{exit_b}} must appear as two consecutive tokens");
            }
            if i + 2 < tokens.len() && matches!(tokens[i + 2], Token::Capture(CaptureName::ExitB)) {
                panic!("entry {entry_name}: {{exit_b}} appears more than twice consecutively");
            }
            // Runtime keeps only the first {exit_b} site but the latest bytes,
            // so multiple pairs would desynchronize. Reject up front.
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

fn has_capture(tokens: &[Token], name: CaptureName) -> bool {
    tokens.iter().any(|t| matches!(t, Token::Capture(n) if *n == name))
}

fn emit_tokens(tokens: &[Token]) -> String {
    let mut s = String::from("&[");
    for t in tokens {
        match t {
            Token::Literal(b) => s.push_str(&format!("PatternToken::Literal({b:#x}), ")),
            Token::AnyByte => s.push_str("PatternToken::AnyByte, "),
            Token::Capture(c) => s.push_str(&format!("PatternToken::Capture(CaptureName::{}), ", c.ident())),
        }
    }
    s.push(']');
    s
}

fn emit_pc_hook(out: &mut String, hook: &RawHook, pc: u32) {
    match hook {
        RawHook::Memcpy { .. } => {
            out.push_str(&format!("            Hook {{ pc: {pc:#x}, kind: HookKind::Memcpy }},\n"));
        }
        RawHook::Memset { .. } => {
            out.push_str(&format!("            Hook {{ pc: {pc:#x}, kind: HookKind::Memset }},\n"));
        }
        RawHook::Strcpy { .. } => {
            out.push_str(&format!("            Hook {{ pc: {pc:#x}, kind: HookKind::Strcpy }},\n"));
        }
        RawHook::Strlen { .. } => {
            out.push_str(&format!("            Hook {{ pc: {pc:#x}, kind: HookKind::Strlen }},\n"));
        }
        RawHook::InlineCopy {
            dst_offset,
            src_offset,
            len_offset,
            exit_pc,
            spill_back,
            ..
        } => {
            let dst = dst_offset.expect("pc-based inline_copy requires dst_offset");
            let src = src_offset.expect("pc-based inline_copy requires src_offset");
            let len = len_offset.expect("pc-based inline_copy requires len_offset");
            let exit = exit_pc.expect("pc-based inline_copy requires exit_pc");
            out.push_str(&format!(
                "            Hook {{ pc: {pc:#x}, kind: HookKind::InlineCopy(InlineCopy {{ dst_offset: {dst}, src_offset: {src}, len_offset: {len}, exit_pc: {exit:#x}, spill_back: {spill_back} }}) }},\n"
            ));
        }
    }
}

fn emit_pattern_hook(out: &mut String, hook: &RawHook, tokens: &[Token], entry_name: &str) {
    let tokens_src = emit_tokens(tokens);
    match hook {
        RawHook::Memcpy { .. } => {
            out.push_str(&format!(
                "            PatternHook {{ tokens: {tokens_src}, kind_template: PatternHookKind::Memcpy }},\n"
            ));
        }
        RawHook::Memset { .. } => {
            out.push_str(&format!(
                "            PatternHook {{ tokens: {tokens_src}, kind_template: PatternHookKind::Memset }},\n"
            ));
        }
        RawHook::Strcpy { .. } => {
            out.push_str(&format!(
                "            PatternHook {{ tokens: {tokens_src}, kind_template: PatternHookKind::Strcpy }},\n"
            ));
        }
        RawHook::Strlen { .. } => {
            out.push_str(&format!(
                "            PatternHook {{ tokens: {tokens_src}, kind_template: PatternHookKind::Strlen }},\n"
            ));
        }
        RawHook::InlineCopy {
            dst_offset,
            src_offset,
            len_offset,
            exit_pc,
            spill_back,
            ..
        } => {
            let dst_cap = has_capture(tokens, CaptureName::Dst);
            let src_cap = has_capture(tokens, CaptureName::Src);
            let len_cap = has_capture(tokens, CaptureName::Len);
            let exit_cap = has_capture(tokens, CaptureName::ExitB);

            // If a capture is absent, the TOML must provide a fixed value (encoded as Some).
            // If a capture is present, runtime fills it — encoded as None.
            let dst_expr = encode_offset("dst_offset", dst_cap, *dst_offset, entry_name);
            let src_expr = encode_offset("src_offset", src_cap, *src_offset, entry_name);
            let len_expr = encode_offset("len_offset", len_cap, *len_offset, entry_name);

            if !exit_cap && exit_pc.is_none() {
                panic!("entry {entry_name}: inline_copy pattern needs either {{exit_b}} capture or exit_pc");
            }
            if exit_cap && exit_pc.is_some() {
                panic!("entry {entry_name}: inline_copy pattern cannot specify both {{exit_b}} and exit_pc");
            }
            let exit_expr = if exit_cap {
                "None".to_string()
            } else {
                format!("Some({:#x})", exit_pc.unwrap())
            };

            out.push_str(&format!(
                "            PatternHook {{ tokens: {tokens_src}, kind_template: PatternHookKind::InlineCopy {{ dst_offset: {dst_expr}, src_offset: {src_expr}, len_offset: {len_expr}, exit_pc: {exit_expr}, spill_back: {spill_back} }} }},\n"
            ));
        }
    }
}

fn encode_offset(field: &str, has_cap: bool, value: Option<i32>, entry_name: &str) -> String {
    match (has_cap, value) {
        (true, None) => "None".to_string(),
        (false, Some(v)) => format!("Some({v})"),
        (true, Some(_)) => panic!("entry {entry_name}: {field} cannot be set when a corresponding capture is in the pattern"),
        (false, None) => panic!("entry {entry_name}: {field} required when no matching capture is in the pattern"),
    }
}

fn main() {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let toml_path = PathBuf::from(manifest_dir).join("../data/native_hooks.toml");
    println!("cargo:rerun-if-changed={}", toml_path.display());

    let text = fs::read_to_string(&toml_path).unwrap_or_else(|e| panic!("read {}: {e}", toml_path.display()));
    let doc: Document = toml::from_str(&text).expect("parse native_hooks.toml");

    let mut out = String::from("&[\n");
    for entry in &doc.entry {
        let hash_bytes = if let Some(h) = &entry.hash {
            let bytes = hex::decode(h).unwrap_or_else(|e| panic!("hex decode {h}: {e}"));
            assert_eq!(bytes.len(), 16, "hash must be 16 bytes, got {}: {h}", bytes.len());
            Some(bytes)
        } else {
            None
        };

        let mut pc_hooks: Vec<&RawHook> = Vec::new();
        let mut pattern_hooks: Vec<(&RawHook, Vec<Token>)> = Vec::new();
        for hook in &entry.hook {
            let (pc, pattern) = match hook {
                RawHook::Memcpy { pc, pattern } => (pc, pattern),
                RawHook::Memset { pc, pattern } => (pc, pattern),
                RawHook::Strcpy { pc, pattern } => (pc, pattern),
                RawHook::Strlen { pc, pattern } => (pc, pattern),
                RawHook::InlineCopy { pc, pattern, .. } => (pc, pattern),
            };
            match (pc.is_some(), pattern.is_some()) {
                (true, false) => pc_hooks.push(hook),
                (false, true) => {
                    let tokens = parse_pattern(pattern.as_ref().unwrap(), &entry.name);
                    validate_exit_b(&tokens, &entry.name);
                    pattern_hooks.push((hook, tokens));
                }
                (true, true) => panic!("entry {}: hook cannot specify both `pc` and `pattern`", entry.name),
                (false, false) => panic!("entry {}: hook must specify either `pc` or `pattern`", entry.name),
            }
        }

        if hash_bytes.is_none() && !pc_hooks.is_empty() {
            panic!(
                "entry {}: hash is required when pc-based hooks are present (a pc only makes sense for a specific binary)",
                entry.name
            );
        }

        out.push_str("    Entry {\n");
        if let Some(bytes) = hash_bytes {
            out.push_str(&format!("        hash: Some({:?}),\n", bytes.as_slice()));
        } else {
            out.push_str("        hash: None,\n");
        }
        out.push_str(&format!("        name: {:?},\n", entry.name));
        out.push_str("        hooks: &[\n");
        for hook in pc_hooks {
            let pc = match hook {
                RawHook::Memcpy { pc, .. } => pc.unwrap(),
                RawHook::Memset { pc, .. } => pc.unwrap(),
                RawHook::Strcpy { pc, .. } => pc.unwrap(),
                RawHook::Strlen { pc, .. } => pc.unwrap(),
                RawHook::InlineCopy { pc, .. } => pc.unwrap(),
            };
            emit_pc_hook(&mut out, hook, pc);
        }
        out.push_str("        ],\n");
        out.push_str("        patterns: &[\n");
        for (hook, tokens) in &pattern_hooks {
            emit_pattern_hook(&mut out, hook, tokens, &entry.name);
        }
        out.push_str("        ],\n");
        out.push_str("    },\n");
    }
    out.push_str("]\n");

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let out_path = PathBuf::from(out_dir).join("native_hooks_data.rs");
    fs::write(&out_path, out).unwrap_or_else(|e| panic!("write {}: {e}", out_path.display()));
}
