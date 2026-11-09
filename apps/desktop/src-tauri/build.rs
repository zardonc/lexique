fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for source in [
        "scripts/dictionary_sample.py",
        "resources/freedict-fra-zho-2025.11.23",
        "scripts/frequency_sample.py",
        "resources/lexique-3.83",
        "scripts/conjugation_sample.py",
        "scripts/conjugation_metadata.py",
        "scripts/conjugation_generation.py",
        "scripts/conjugation_syntax.py",
        "scripts/conjugation_meanings.py",
        "scripts/conjugation_pronominal.py",
        "scripts/conjugation_runtime.py",
        "scripts/conjugation_coloring.py",
        "scripts/conjugation_endings.json",
        "resources/lefff-3.4",
        "resources/grammalecte-2.3.0",
        "resources/lefff-syntax",
        "resources/freedict-fra-eng-0.4.1",
        "LICENSE",
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
    let prepared = std::process::Command::new("python")
        .args([
            "-X",
            "utf8",
            "scripts/conjugation_sample.py",
            "prepare",
            "target/desktop-conjugation-v6-stats",
        ])
        .current_dir(&root)
        .output()
        .expect("Python 3.12+ is required to prepare bundled conjugations");
    assert!(
        prepared.status.success(),
        "Bundled conjugation preparation failed: {}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    let conjugation: serde_json::Value =
        serde_json::from_slice(&prepared.stdout).expect("Invalid conjugation report");
    println!(
        "cargo:rustc-env=LEXIQUE_CONJUGATION_SHA256={}",
        conjugation["sqlite_sha256"]
            .as_str()
            .expect("Missing conjugation hash")
    );
    let prepared = std::process::Command::new("python")
        .args([
            "-B",
            "-X",
            "utf8",
            "scripts/frequency_sample.py",
            "target/desktop-frequency-v1",
        ])
        .current_dir(&root)
        .output()
        .expect("Python 3.12+ is required to prepare bundled frequencies");
    assert!(
        prepared.status.success(),
        "Bundled frequency preparation failed: {}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    let frequency: serde_json::Value =
        serde_json::from_slice(&prepared.stdout).expect("Invalid frequency report");
    println!(
        "cargo:rustc-env=LEXIQUE_FREQUENCY_SHA256={}",
        frequency["sqlite_sha256"]
            .as_str()
            .expect("Missing frequency hash")
    );
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "runtime_status",
            "dictionary_lookup",
            "dictionary_settings",
            "dictionary_import",
            "dictionary_remove",
            "frequency_lookup",
            "selection_capture",
            "selection_cancel",
            "selection_shortcut",
            "ocr_shortcut",
            "ocr_image",
            "ocr_capture",
            "ocr_locale",
            "ocr_screen",
            "ocr_region",
            "ocr_abort",
            "conjugation_lookup",
            "conjugation_preferences",
            "conjugation_block_order",
            "vocabulary_find",
            "vocabulary_list",
            "vocabulary_save",
            "vocabulary_sort",
            "review_add",
            "review_add_batch",
            "review_batch_status",
            "review_cancel_batch",
            "speech_retry_missing",
            "review_pause",
            "review_auto_enroll",
            "review_cards",
            "review_availability",
            "review_progress",
            "review_rate",
            "review_undo_target",
            "review_undo",
            "review_queue",
            "review_settings",
            "speech_cached",
            "speech_generate",
            "speech_preview",
            "speech_preferences",
            "finish_close",
        ]),
    ))
    .expect("Tauri build metadata could not be generated");
}
