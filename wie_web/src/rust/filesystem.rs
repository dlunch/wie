use alloc::{boxed::Box, rc::Rc, string::ToString, vec};
use core::cell::RefCell;
use core::cmp::{max, min};

use wie_backend::Filesystem;

use crate::indexed_db_store::{Store, StoreKey};

const DB_NAME: &str = "wie_filesystem";
const STORE_NAME: &str = "files";

fn make_key(aid: &str, path: &str) -> StoreKey {
    StoreKey::Pair(aid.to_string(), path.to_string())
}

pub struct WebFilesystem {
    store: Rc<RefCell<Option<Store>>>,
}

// single threaded wasm; RefCell + Rc are only touched sequentially.
unsafe impl Send for WebFilesystem {}
unsafe impl Sync for WebFilesystem {}

impl WebFilesystem {
    pub fn new() -> Self {
        Self {
            store: Rc::new(RefCell::new(None)),
        }
    }

    async fn store(&self) -> Store {
        if let Some(store) = self.store.borrow().as_ref() {
            return store.clone();
        }
        let store = Store::open(DB_NAME, STORE_NAME).await;
        *self.store.borrow_mut() = Some(store.clone());
        store
    }
}

#[async_trait::async_trait]
impl Filesystem for WebFilesystem {
    async fn exists(&self, aid: &str, path: &str) -> bool {
        self.store().await.get(make_key(aid, path)).await.is_some()
    }

    async fn size(&self, aid: &str, path: &str) -> Option<usize> {
        self.store().await.get(make_key(aid, path)).await.map(|v| v.len())
    }

    async fn read(&self, aid: &str, path: &str, offset: usize, count: usize, buf: &mut [u8]) -> Option<usize> {
        let data = self.store().await.get(make_key(aid, path)).await?;

        if offset >= data.len() {
            return Some(0);
        }

        let size_to_read = min(count, data.len() - offset);
        buf[..size_to_read].copy_from_slice(&data[offset..offset + size_to_read]);
        Some(size_to_read)
    }

    async fn write(&self, aid: &str, path: &str, offset: usize, data: &[u8]) -> usize {
        let key = make_key(aid, path);
        let store = self.store().await;
        let existing = store.get(key.clone()).await.unwrap_or_default();
        let new_len = max(existing.len(), offset + data.len());

        let mut next = vec![0u8; new_len];
        next[..existing.len()].copy_from_slice(&existing);
        next[offset..offset + data.len()].copy_from_slice(data);

        store.set(key, &next).await;
        data.len()
    }

    async fn truncate(&self, aid: &str, path: &str, len: usize) {
        let key = make_key(aid, path);
        let store = self.store().await;
        let existing = store.get(key.clone()).await.unwrap_or_default();

        let mut next = vec![0u8; len];
        let copy_len = min(existing.len(), len);
        next[..copy_len].copy_from_slice(&existing[..copy_len]);

        store.set(key, &next).await;
    }
}
