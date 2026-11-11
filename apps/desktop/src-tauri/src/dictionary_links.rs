use lexique_dictionary::DictionaryError;
use serde::Deserialize;

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
pub async fn dictionary_resource_open(id: String) -> Result<(), DictionaryError> {
    let url = page(&id)?;
    tauri::async_runtime::spawn_blocking(move || open(&url))
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
        assert_eq!(pages.len(), 12);
        for item in &pages {
            assert_eq!(pages.iter().filter(|other| other.id == item.id).count(), 1);
            assert!(item.url.starts_with("https://"));
        }
    }
}
