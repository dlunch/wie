pub mod init;
mod java;
mod stdlib;
mod svc_ids;
mod wipi_c;

const SVC_CATEGORY_INIT: u32 = 1;
const SVC_CATEGORY_WIPIC: u32 = 3;
const SVC_CATEGORY_STDLIB: u32 = 5;
/// Native -> platform method trampolines (the `java_load_classes` offset tables).
const SVC_CATEGORY_JAVA_TRAMPOLINE: u32 = 7;
/// Java-interface imports (table `0x64`) routed by `function_index` so each keeps
/// its identity (the SVC id *is* the import index). Lets unknown imports be logged
/// with their index and specific ones (e.g. the native String factory) implemented.
const SVC_CATEGORY_JAVA_INTERFACE: u32 = 9;
