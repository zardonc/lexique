use lexique_dictionary::{CORE_DICTIONARY_ID, DictionaryError};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tauri::Manager;

const CORE_MATERIALS: &[&str] = &[
    "core.sqlite3",
    "report.json",
    "COPYING",
    "LICENSE-FranceTerme.pdf",
    "NOTICES.md",
    "README.md",
    "curated-inputs.json",
    "field-review.json",
    "scope-manifest.csv",
    "selected-source-context.json",
    "selected-source-excerpts.json",
    "MEMBERSHIP-NOTICES.txt",
    "LEXIQUE-UPSTREAM-README.txt",
    "lexique-provenance.json",
    "input-checksums.json",
    "coverage.json",
    "core_pilot.py",
    "LICENSE-MIT",
];

#[derive(Deserialize)]
struct Page {
    id: String,
    url: String,
}

fn failure(code: &'static str, message: impl std::fmt::Display) -> DictionaryError {
    DictionaryError {
        code,
        message: message.to_string(),
    }
}

fn page(id: &str) -> Result<String, DictionaryError> {
    if id.len() > 64 {
        return Err(failure("invalid_query", "Unknown dictionary resource"));
    }
    let pages: Vec<Page> =
        serde_json::from_str(include_str!("../../src/dictionary-resources.json"))
            .map_err(|error| failure("corrupt", error))?;
    pages
        .into_iter()
        .find(|page| page.id == id)
        .map(|page| page.url)
        .ok_or_else(|| failure("invalid_query", "Unknown dictionary resource"))
}

fn bundled_materials(resource_dir: &Path) -> Result<PathBuf, DictionaryError> {
    let folder = resource_dir.join("dictionary/core-pilot-79");
    for &name in CORE_MATERIALS {
        if !folder.join(name).is_file() {
            return Err(failure(
                "missing",
                format!(
                    "Bundled core material is missing: {}",
                    folder.join(name).display()
                ),
            ));
        }
    }
    Ok(folder)
}

#[cfg(target_os = "windows")]
fn open(url: &str) -> Result<(), DictionaryError> {
    use windows::{
        Win32::{
            System::Com::{
                COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
            },
            UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
        },
        core::{HSTRING, w},
    };
    let target = HSTRING::from(url);
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok() }
        .map_err(|error| failure("query_failed", error))?;
    let result = unsafe { ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL) };
    unsafe {
        CoUninitialize();
    }
    let code = result.0 as isize;
    if code <= 32 {
        return Err(failure(
            "query_failed",
            format!("Windows could not open the source page (Shell error {code})"),
        ));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn open(url: &str) -> Result<(), DictionaryError> {
    let status = std::process::Command::new("open")
        .arg(url)
        .status()
        .map_err(|error| failure("query_failed", error))?;
    if !status.success() {
        return Err(failure(
            "query_failed",
            format!("Source-page launcher exited with {status}"),
        ));
    }
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn open(_url: &str) -> Result<(), DictionaryError> {
    Err(failure(
        "unsupported",
        "Opening resource pages is unavailable on this platform",
    ))
}

#[tauri::command]
pub async fn dictionary_resource_open(
    app: tauri::AppHandle,
    id: String,
) -> Result<(), DictionaryError> {
    let target = if id == CORE_DICTIONARY_ID {
        let resource_dir = app.path().resource_dir().map_err(|error| {
            failure(
                "missing",
                format!("Could not resolve bundled sources: {error}"),
            )
        })?;
        bundled_materials(&resource_dir)?
            .to_str()
            .ok_or_else(|| failure("query_failed", "Bundled source path is not valid UTF-8"))?
            .to_owned()
    } else {
        page(&id)?
    };
    tauri::async_runtime::spawn_blocking(move || open(&target))
        .await
        .map_err(|error| failure("query_failed", error))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_catalogue_ids_select_https_pages() {
        assert_eq!(
            page("collins-fr-en").unwrap(),
            "https://forum.freemdict.com/t/topic/31704"
        );
        for id in [
            "file:///C:/Windows",
            "https://example.org",
            "../lexique.sqlite3",
            "",
        ] {
            assert_eq!(page(id).unwrap_err().code, "invalid_query");
        }
        let pages: Vec<Page> =
            serde_json::from_str(include_str!("../../src/dictionary-resources.json")).unwrap();
        assert_eq!(pages.len(), 13);
        for item in &pages {
            assert_eq!(pages.iter().filter(|other| other.id == item.id).count(), 1);
            assert!(item.url.starts_with("https://"));
        }
    }

    #[test]
    fn core_sources_resolve_only_inside_the_bundle_and_require_real_materials() {
        let root = std::env::temp_dir().join(format!(
            "lexique-core-notices-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let folder = root.join("dictionary/core-pilot-79");
        std::fs::create_dir_all(&folder).unwrap();
        assert_eq!(bundled_materials(&root).unwrap_err().code, "missing");
        for name in CORE_MATERIALS {
            std::fs::write(folder.join(name), b"bundled test material").unwrap();
        }
        assert_eq!(bundled_materials(&root).unwrap(), folder);
        std::fs::remove_file(folder.join("COPYING")).unwrap();
        let error = bundled_materials(&root).unwrap_err();
        assert_eq!(error.code, "missing");
        assert!(error.message.contains("COPYING"));
        std::fs::write(folder.join("COPYING"), b"bundled test material").unwrap();
        std::fs::remove_file(folder.join("LICENSE-FranceTerme.pdf")).unwrap();
        let error = bundled_materials(&root).unwrap_err();
        assert_eq!(error.code, "missing");
        assert!(error.message.contains("LICENSE-FranceTerme.pdf"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
