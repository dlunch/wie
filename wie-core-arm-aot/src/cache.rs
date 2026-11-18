use alloc::vec::Vec;
use core::ops::Range;

use nom::{Parser, bytes::complete::take, number::complete::le_u32};

use crate::CompileRequest;

/// Increment when analysis, generated code, execution ABI, or the cache format changes.
pub const AOT_CACHE_VERSION: u32 = 2;
const MAGIC: &[u8; 8] = b"WIEAOT\0\0";

pub fn key_input(request: &CompileRequest) -> Vec<u8> {
    let mut input = Vec::new();
    input.extend_from_slice(&AOT_CACHE_VERSION.to_le_bytes());
    for value in [request.max_region_instructions, request.max_region_blocks, request.images.len()] {
        input.extend_from_slice(&(value as u64).to_le_bytes());
    }
    for image in &*request.images {
        input.extend_from_slice(&image.address.to_le_bytes());
        input.extend_from_slice(&(image.bytes.len() as u64).to_le_bytes());
        input.extend_from_slice(&image.bytes);
    }
    input
}

pub const HEADER_LENGTH: usize = 52;

/// A record is this header followed by the code and manifest; hosts append a SHA-256 digest before storing it.
/// Hosts write the code next to the header themselves so the module is never copied into a second buffer.
pub fn encode_header(key: &[u8; 32], code_length: usize, manifest_length: usize) -> Option<Vec<u8>> {
    let code_length = u32::try_from(code_length).ok()?;
    let manifest_length = u32::try_from(manifest_length).ok()?;
    let mut bytes = Vec::with_capacity(HEADER_LENGTH);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&AOT_CACHE_VERSION.to_le_bytes());
    bytes.extend_from_slice(key);
    bytes.extend_from_slice(&code_length.to_le_bytes());
    bytes.extend_from_slice(&manifest_length.to_le_bytes());
    Some(bytes)
}

pub fn encode(key: &[u8; 32], code: &[u8], manifest: &[u8]) -> Option<Vec<u8>> {
    let mut bytes = encode_header(key, code.len(), manifest.len())?;
    bytes.extend_from_slice(code);
    bytes.extend_from_slice(manifest);
    Some(bytes)
}

/// Validate a record header against the record length (digest already removed) and return the code and manifest ranges.
pub fn decode_header(header: &[u8], record_length: usize, key: &[u8; 32]) -> Option<(Range<usize>, Range<usize>)> {
    let parsed: nom::IResult<_, _> = (take(8usize), le_u32, take(32usize), le_u32, le_u32).parse(header);
    let (_, (magic, version, stored_key, code_length, manifest_length)) = parsed.ok()?;
    if magic != MAGIC || version != AOT_CACHE_VERSION || stored_key != key {
        return None;
    }
    let code_end = HEADER_LENGTH.checked_add(code_length as usize)?;
    let manifest_end = code_end.checked_add(manifest_length as usize)?;
    (manifest_end == record_length).then_some((HEADER_LENGTH..code_end, code_end..manifest_end))
}

/// Decode a payload after the host has verified and removed its SHA-256 digest.
pub fn decode<'a>(bytes: &'a [u8], key: &[u8; 32]) -> Option<(&'a [u8], &'a [u8])> {
    let (code, manifest) = decode_header(bytes, bytes.len(), key)?;
    Some((&bytes[code], &bytes[manifest]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_roundtrip_rejects_truncation_and_incompatible_records() {
        let key = [7; 32];
        let encoded = encode(&key, b"code", b"manifest").unwrap();
        assert_eq!(decode(&encoded, &key), Some((&b"code"[..], &b"manifest"[..])));
        assert!(decode(&encoded, &[8; 32]).is_none());
        for length in 0..encoded.len() {
            assert!(decode(&encoded[..length], &key).is_none());
        }
        for (offset, value) in [(0, 0), (8, AOT_CACHE_VERSION + 1), (44, u32::MAX), (48, 0)] {
            let mut corrupt = encoded.clone();
            corrupt[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(decode(&corrupt, &key).is_none());
        }
        let empty = encode(&key, &[], &[]).unwrap();
        assert_eq!(decode(&empty, &key), Some((&[][..], &[][..])));
    }
}
