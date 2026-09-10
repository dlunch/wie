use alloc::string::ToString;

use js_sys::{Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use wie_arm_jit_types::CompileRequest;

#[wasm_bindgen]
pub fn compile_request(payload: &[u8]) -> Result<JsValue, JsValue> {
    let request: CompileRequest = serde_json::from_slice(payload).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let artifact = crate::codegen::compile(&request).map_err(|error| JsValue::from_str(&error))?;
    let manifest = serde_json::to_string(&artifact.manifest).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let result = Object::new();
    Reflect::set(&result, &"bytes".into(), &Uint8Array::from(artifact.bytes.as_slice()))?;
    Reflect::set(&result, &"manifest".into(), &manifest.into())?;
    Ok(result.into())
}
