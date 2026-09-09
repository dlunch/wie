#![no_std]

use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOCATOR: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    core::arch::wasm32::unreachable()
}

#[wasm_bindgen]
pub fn compile_request(payload: &str) -> Result<JsValue, JsValue> {
    wie_arm_wasm::compiler::compile_request(payload)
}
