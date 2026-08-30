use alloc::{borrow::ToOwned, string::String, vec::Vec};

use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;

use crate::util::run_js_future;

#[wasm_bindgen(module = "/src/ts/indexed_db_store.ts")]
extern "C" {
    type IndexedDBStore;

    #[wasm_bindgen(static_method_of = IndexedDBStore)]
    async fn open(db_name: &str, store_name: &str) -> IndexedDBStore;

    #[wasm_bindgen(method)]
    async fn get_all_keys(this: &IndexedDBStore) -> js_sys::Array;

    #[wasm_bindgen(method)]
    async fn get(this: &IndexedDBStore, key: &JsValue) -> JsValue;

    #[wasm_bindgen(method)]
    async fn set(this: &IndexedDBStore, key: &JsValue, data: Uint8Array);

    #[wasm_bindgen(method)]
    async fn delete(this: &IndexedDBStore, key: &JsValue);
}

unsafe impl Sync for IndexedDBStore {}
unsafe impl Send for IndexedDBStore {}

pub struct Store {
    js: IndexedDBStore,
}

#[derive(Clone)]
pub enum StoreKey {
    String(String),
    Pair(String, String),
}

impl StoreKey {
    fn into_js_value(self) -> JsValue {
        match self {
            Self::String(value) => JsValue::from_str(&value),
            Self::Pair(first, second) => Array::of2(&JsValue::from_str(&first), &JsValue::from_str(&second)).into(),
        }
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Self { js: self.js.clone().into() }
    }
}

impl Store {
    pub async fn open(db_name: &str, store_name: &str) -> Self {
        let db_name = db_name.to_owned();
        let store_name = store_name.to_owned();
        let js = run_js_future(async move { IndexedDBStore::open(&db_name, &store_name).await }).await;
        Self { js }
    }

    pub async fn get_all_keys(&self) -> Vec<String> {
        let js: IndexedDBStore = self.js.clone().into();
        run_js_future(async move { js.get_all_keys().await.iter().filter_map(|key| key.as_string()).collect() }).await
    }

    pub async fn get(&self, key: StoreKey) -> Option<Vec<u8>> {
        let js: IndexedDBStore = self.js.clone().into();
        run_js_future(async move {
            let data = js.get(&key.into_js_value()).await;
            if data.is_undefined() {
                None
            } else {
                Some(Uint8Array::from(data).to_vec())
            }
        })
        .await
    }

    pub async fn set(&self, key: StoreKey, data: &[u8]) {
        let js: IndexedDBStore = self.js.clone().into();
        let data = data.to_vec();
        run_js_future(async move {
            let array = Uint8Array::from(data.as_slice());
            js.set(&key.into_js_value(), array).await;
        })
        .await;
    }

    pub async fn delete(&self, key: StoreKey) {
        let js: IndexedDBStore = self.js.clone().into();
        run_js_future(async move { js.delete(&key.into_js_value()).await }).await;
    }
}
