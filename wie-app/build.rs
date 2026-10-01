fn main() {
    tauri_build::build();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android")
        && let Some(project) = std::env::var_os("TAURI_ANDROID_PROJECT_PATH")
    {
        let package = std::path::PathBuf::from(project).join("app/src/main/java/net/dlunch/wie");
        for source in ["audio/NativeAudio.kt", "screen/NativeScreenView.kt"] {
            let original = format!("android/src/main/java/net/dlunch/wie/{source}");
            println!("cargo:rerun-if-changed={original}");
            let destination = package.join(source);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::copy(original, destination).unwrap();
        }
    }
}
