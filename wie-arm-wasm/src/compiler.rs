use alloc::string::ToString;

use js_sys::{Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use wie_arm_jit::CompileRequest;

pub fn compile_request(payload: &str) -> Result<JsValue, JsValue> {
    let request: CompileRequest = serde_json::from_str(payload).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let artifact = crate::codegen::compile(&request).map_err(|error| JsValue::from_str(&error))?;
    let manifest = serde_json::to_string(&artifact.manifest).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let result = Object::new();
    Reflect::set(&result, &"bytes".into(), &Uint8Array::from(artifact.bytes.as_slice()))?;
    Reflect::set(&result, &"manifest".into(), &manifest.into())?;
    Ok(result.into())
}
