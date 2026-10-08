use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::RecordId;

#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum StorageRequest<S = String, B = Vec<u8>> {
    FileExists { aid: S, path: S },
    FileSize { aid: S, path: S },
    FileRead { aid: S, path: S, offset: usize, count: usize },
    FileWrite { aid: S, path: S, offset: usize, data: B },
    FileTruncate { aid: S, path: S, length: usize },
    DbOpen { pid: S, name: S },
    DbExists { pid: S, name: S },
    DbDelete { pid: S, name: S },
    DbUsage { pid: S },
    RecordNextId { pid: S, name: S },
    RecordIds { pid: S, name: S },
    RecordGet { pid: S, name: S, id: RecordId },
    RecordDelete { pid: S, name: S, id: RecordId },
    RecordAdd { pid: S, name: S, data: B },
    RecordSet { pid: S, name: S, id: RecordId, data: B },
}
