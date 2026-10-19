// Temporary browser save migration added 2026-10-19.
use alloc::string::{String, ToString};

use js_sys::Function;
use wasm_bindgen::prelude::*;
use wie_backend::normalize_guest_path;

#[wasm_bindgen(module = "/src/ts/save_migration.ts")]
extern "C" {
    #[wasm_bindgen(catch, js_name = migrateSaves)]
    async fn migrate_saves(pid: &str, aid: &str, clet: bool, normalize: &Function) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(js_name = migrateSaves)]
pub async fn run(filename: &str, bytes: &[u8], pid: &str) -> Result<(), JsValue> {
    let (aid, clet) = wie::save_migration_identity(filename, bytes, pid).map_err(|error| JsError::new(&error.to_string()))?;
    let normalize = Closure::<dyn Fn(String) -> Option<String>>::new(|path: String| normalize_guest_path(&path));
    migrate_saves(pid, &aid, clet, normalize.as_ref().unchecked_ref()).await?;
    Ok(())
}
