fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for source in [
        "scripts/dictionary_sample.py",
        "resources/freedict-fra-zho-2025.11.23",
    ] {
        println!("cargo:rerun-if-changed={}", root.join(source).display());
    }
    let status = std::process::Command::new("python")
        .args([
            "-X",
            "utf8",
            "scripts/dictionary_sample.py",
            "prepare",
            "target/desktop-dictionary-sample",
        ])
        .current_dir(&root)
        .status()
        .expect("Python 3.12+ is required to prepare the bundled dictionary");
    assert!(status.success(), "Bundled dictionary preparation failed");
    let report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("target/desktop-dictionary-sample/report.json"))
            .expect("Missing dictionary report"),
    )
    .expect("Invalid dictionary report");
    println!(
        "cargo:rustc-env=LEXIQUE_DICTIONARY_SHA256={}",
        report["sqlite_sha256"]
            .as_str()
            .expect("Missing dictionary hash")
    );
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&["runtime_status", "dictionary_lookup"]),
    ))
    .expect("Tauri build metadata could not be generated");
}
