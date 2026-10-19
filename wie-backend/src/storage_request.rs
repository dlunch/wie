use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::RecordId;

#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum StorageRequest<S = String, B = Vec<u8>> {
    FileExists { pid: S, path: S },
    FileSize { pid: S, path: S },
    FileRead { pid: S, path: S, offset: usize, count: usize },
    FileWrite { pid: S, path: S, offset: usize, data: B },
    FileTruncate { pid: S, path: S, length: usize },
    FileRemove { pid: S, path: S },
    FileUsage { pid: S },
    DbOpen { pid: S, name: S },
    DbExists { pid: S, name: S },
    DbDelete { pid: S, name: S },
    RecordNextId { pid: S, name: S },
    RecordIds { pid: S, name: S },
    RecordGet { pid: S, name: S, id: RecordId },
    RecordDelete { pid: S, name: S, id: RecordId },
    RecordAdd { pid: S, name: S, data: B },
    RecordSet { pid: S, name: S, id: RecordId, data: B },
}
