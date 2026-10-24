fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["runtime_status"])),
    )
    .expect("Tauri build metadata could not be generated");
}
