use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use core::future::Future;

use js_sys::JSON;
use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;

use wie_backend::{Database, DatabaseRepository, Filesystem, RecordId};

use crate::{audio_sink::AudioPlayer, util::run_js_future};

#[wasm_bindgen]
extern "C" {
    #[derive(Clone)]
    #[wasm_bindgen(typescript_type = "import('../src/ts/backend/tauri-host').EmulatorHost")]
    pub type WasmHost;

    #[wasm_bindgen(method, structural, catch)]
    async fn storage(this: &WasmHost, request: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, structural, getter)]
    pub fn audio(this: &WasmHost) -> AudioPlayer;

    #[wasm_bindgen(method, structural)]
    pub fn now(this: &WasmHost) -> f64;

    #[wasm_bindgen(method, structural)]
    pub fn vibrate(this: &WasmHost, duration_ms: f64, intensity: u8);
}

// The Wasm runtime and its JavaScript host run on one thread.
unsafe impl Send for WasmHost {}
unsafe impl Sync for WasmHost {}

#[derive(Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum StorageRequest<'a> {
    FileExists {
        aid: &'a str,
        path: &'a str,
    },
    FileSize {
        aid: &'a str,
        path: &'a str,
    },
    FileRead {
        aid: &'a str,
        path: &'a str,
        offset: usize,
        count: usize,
    },
    FileWrite {
        aid: &'a str,
        path: &'a str,
        offset: usize,
        data: &'a [u8],
    },
    FileTruncate {
        aid: &'a str,
        path: &'a str,
        length: usize,
    },
    DbOpen {
        pid: &'a str,
        name: &'a str,
    },
    DbExists {
        pid: &'a str,
        name: &'a str,
    },
    DbDelete {
        pid: &'a str,
        name: &'a str,
    },
    DbUsage {
        pid: &'a str,
    },
    RecordNextId {
        pid: &'a str,
        name: &'a str,
    },
    RecordIds {
        pid: &'a str,
        name: &'a str,
    },
    RecordGet {
        pid: &'a str,
        name: &'a str,
        id: RecordId,
    },
    RecordDelete {
        pid: &'a str,
        name: &'a str,
        id: RecordId,
    },
    RecordAdd {
        pid: &'a str,
        name: &'a str,
        data: &'a [u8],
    },
    RecordSet {
        pid: &'a str,
        name: &'a str,
        id: RecordId,
        data: &'a [u8],
    },
}

impl WasmHost {
    fn request<T: DeserializeOwned + Default + 'static>(&self, request: StorageRequest<'_>) -> impl Future<Output = T> + Send + use<T> {
        let host = self.clone();
        // Own the payload before yielding; guest buffers may change while IPC is pending.
        let request = serde_json::to_string(&request);
        run_js_future(async move {
            let result: Result<T, JsValue> = async {
                let json = request.map_err(|error| JsValue::from_str(&error.to_string()))?;
                let request = JSON::parse(&json)?;
                let response = host.storage(&request).await?;
                let json = String::from(JSON::stringify(&response)?);
                serde_json::from_str(&json).map_err(|error| JsValue::from_str(&error.to_string()))
            }
            .await;

            result.unwrap_or_else(|error| {
                // Active host rejection has already initiated terminal cleanup in JS.
                // Infallible guest traits settle without reporting a successful write.
                tracing::warn!(?error, "Host storage request failed");
                T::default()
            })
        })
    }
}

#[async_trait::async_trait]
impl DatabaseRepository for WasmHost {
    async fn open(&self, name: &str, app_id: &str) -> Box<dyn Database> {
        self.request::<()>(StorageRequest::DbOpen { pid: app_id, name }).await;
        Box::new(HostDatabase {
            host: self.clone(),
            pid: app_id.to_string(),
            name: name.to_string(),
        })
    }

    async fn exists(&self, name: &str, app_id: &str) -> bool {
        self.request(StorageRequest::DbExists { pid: app_id, name }).await
    }

    async fn delete(&self, name: &str, app_id: &str) -> bool {
        self.request(StorageRequest::DbDelete { pid: app_id, name }).await
    }

    async fn usage(&self, app_id: &str) -> u64 {
        self.request(StorageRequest::DbUsage { pid: app_id }).await
    }
}

struct HostDatabase {
    host: WasmHost,
    pid: String,
    name: String,
}

#[async_trait::async_trait]
impl Database for HostDatabase {
    async fn next_id(&self) -> RecordId {
        self.host
            .request(StorageRequest::RecordNextId {
                pid: &self.pid,
                name: &self.name,
            })
            .await
    }

    async fn add(&mut self, data: &[u8]) -> RecordId {
        self.host
            .request(StorageRequest::RecordAdd {
                pid: &self.pid,
                name: &self.name,
                data,
            })
            .await
    }

    async fn get(&self, id: RecordId) -> Option<Vec<u8>> {
        self.host
            .request(StorageRequest::RecordGet {
                pid: &self.pid,
                name: &self.name,
                id,
            })
            .await
    }

    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool {
        self.host
            .request(StorageRequest::RecordSet {
                pid: &self.pid,
                name: &self.name,
                id,
                data,
            })
            .await
    }

    async fn delete(&mut self, id: RecordId) -> bool {
        self.host
            .request(StorageRequest::RecordDelete {
                pid: &self.pid,
                name: &self.name,
                id,
            })
            .await
    }

    async fn get_record_ids(&self) -> Vec<RecordId> {
        self.host
            .request(StorageRequest::RecordIds {
                pid: &self.pid,
                name: &self.name,
            })
            .await
    }
}

#[async_trait::async_trait]
impl Filesystem for WasmHost {
    async fn exists(&self, aid: &str, path: &str) -> bool {
        self.request(StorageRequest::FileExists { aid, path }).await
    }

    async fn size(&self, aid: &str, path: &str) -> Option<usize> {
        self.request(StorageRequest::FileSize { aid, path }).await
    }

    async fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let data: Option<Vec<u8>> = self.request(StorageRequest::FileRead { aid, path, offset, count }).await;
        let data = data?;
        let len = data.len().min(count);
        buf[..len].copy_from_slice(&data[..len]);
        Some(len)
    }

    async fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        self.request(StorageRequest::FileWrite { aid, path, offset, data }).await
    }

    async fn truncate(&self, aid: &str, path: &str, length: usize) {
        self.request::<()>(StorageRequest::FileTruncate { aid, path, length }).await;
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, to_value};

    use super::StorageRequest;

    #[test]
    fn storage_requests_preserve_namespaces_offsets_and_empty_data() {
        let requests = [
            (
                StorageRequest::FileExists { aid: "app", path: "save" },
                json!({"op": "fileExists", "aid": "app", "path": "save"}),
            ),
            (
                StorageRequest::FileSize { aid: "app", path: "save" },
                json!({"op": "fileSize", "aid": "app", "path": "save"}),
            ),
            (
                StorageRequest::FileRead {
                    aid: "app",
                    path: "save",
                    offset: 2,
                    count: 4,
                },
                json!({"op": "fileRead", "aid": "app", "path": "save", "offset": 2, "count": 4}),
            ),
            (
                StorageRequest::FileWrite {
                    aid: "app",
                    path: "save",
                    offset: 3,
                    data: &[0, 255],
                },
                json!({"op": "fileWrite", "aid": "app", "path": "save", "offset": 3, "data": [0, 255]}),
            ),
            (
                StorageRequest::FileTruncate {
                    aid: "app",
                    path: "save",
                    length: 0,
                },
                json!({"op": "fileTruncate", "aid": "app", "path": "save", "length": 0}),
            ),
            (
                StorageRequest::DbOpen {
                    pid: "owner",
                    name: "records",
                },
                json!({"op": "dbOpen", "pid": "owner", "name": "records"}),
            ),
            (
                StorageRequest::DbExists {
                    pid: "owner",
                    name: "records",
                },
                json!({"op": "dbExists", "pid": "owner", "name": "records"}),
            ),
            (
                StorageRequest::DbDelete {
                    pid: "owner",
                    name: "records",
                },
                json!({"op": "dbDelete", "pid": "owner", "name": "records"}),
            ),
            (StorageRequest::DbUsage { pid: "owner" }, json!({"op": "dbUsage", "pid": "owner"})),
            (
                StorageRequest::RecordNextId {
                    pid: "owner",
                    name: "records",
                },
                json!({"op": "recordNextId", "pid": "owner", "name": "records"}),
            ),
            (
                StorageRequest::RecordIds {
                    pid: "owner",
                    name: "records",
                },
                json!({"op": "recordIds", "pid": "owner", "name": "records"}),
            ),
            (
                StorageRequest::RecordGet {
                    pid: "owner",
                    name: "records",
                    id: 7,
                },
                json!({"op": "recordGet", "pid": "owner", "name": "records", "id": 7}),
            ),
            (
                StorageRequest::RecordDelete {
                    pid: "owner",
                    name: "records",
                    id: 7,
                },
                json!({"op": "recordDelete", "pid": "owner", "name": "records", "id": 7}),
            ),
            (
                StorageRequest::RecordAdd {
                    pid: "owner",
                    name: "records",
                    data: &[],
                },
                json!({"op": "recordAdd", "pid": "owner", "name": "records", "data": []}),
            ),
            (
                StorageRequest::RecordSet {
                    pid: "owner",
                    name: "records",
                    id: 7,
                    data: &[],
                },
                json!({"op": "recordSet", "pid": "owner", "name": "records", "id": 7, "data": []}),
            ),
        ];

        for (request, expected) in requests {
            assert_eq!(to_value(request).unwrap(), expected);
        }
    }
}
