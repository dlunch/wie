use std::env;

fn main() {
    let mut attributes = tauri_build::Attributes::new();
    if env::var("CARGO_CFG_TARGET_ENV").unwrap() == "msvc" {
        // Tauri's resource manifest only reaches app binaries, so embed it for tests too.
        attributes = attributes.windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTUAC:NO");
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' \
             version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
    tauri_build::try_build(attributes).expect("failed to run Tauri build");
}
