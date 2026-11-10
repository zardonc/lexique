use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc,
};
use std::time::Duration;

use serde::Serialize;
use tauri::{Emitter, Manager};

#[cfg(windows)]
#[path = "selection_windows.rs"]
mod platform;
#[cfg(target_os = "macos")]
#[path = "selection_macos.rs"]
mod platform;

pub const MAX_TEXT: usize = 5_000;

#[derive(Default)]
pub struct SelectionState {
    sequence: AtomicU64,
    busy: AtomicBool,
}

#[derive(Debug)]
pub struct SelectionError {
    pub code: &'static str,
    pub message: String,
}

impl SelectionError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SelectionEvent {
    Pending {
        request: u64,
    },
    Cancelled {
        request: u64,
    },
    Captured {
        request: u64,
        text: String,
    },
    Image {
        request: u64,
        image: crate::ocr::OcrImage,
        recognize: bool,
    },
    Failed {
        request: u64,
        code: &'static str,
        message: String,
    },
}

pub fn validate(text: String) -> Result<String, SelectionError> {
    let text: String = text.chars().take(MAX_TEXT).collect();
    if text.trim().is_empty() {
        return Err(SelectionError::new("no_selection", ""));
    }
    Ok(text)
}

pub fn current(app: &tauri::AppHandle, request: u64) -> bool {
    !app.state::<crate::vocabulary::CloseAuthorization>()
        .permits_exit()
        && app
            .state::<SelectionState>()
            .sequence
            .load(Ordering::SeqCst)
            == request
}

pub fn begin(app: &tauri::AppHandle) -> Result<u64, String> {
    if app
        .state::<crate::vocabulary::CloseAuthorization>()
        .permits_exit()
    {
        return Err("Application is closing".into());
    }
    crate::ocr_capture::clear(app)?;
    let request = app
        .state::<SelectionState>()
        .sequence
        .fetch_add(1, Ordering::SeqCst)
        + 1;
    app.emit_to(
        "main",
        "selection-input",
        SelectionEvent::Pending { request },
    )
    .map_err(|error| error.to_string())?;
    Ok(request)
}

pub fn deliver(app: &tauri::AppHandle, event: SelectionEvent) -> Result<(), String> {
    let request = match &event {
        SelectionEvent::Pending { request }
        | SelectionEvent::Cancelled { request }
        | SelectionEvent::Captured { request, .. }
        | SelectionEvent::Image { request, .. }
        | SelectionEvent::Failed { request, .. } => *request,
    };
    if !current(app, request) {
        return Ok(());
    }
    let window = app
        .get_webview_window("main")
        .ok_or("Main window is unavailable")?;
    let focus = window
        .show()
        .map_err(|error| format!("Could not show input: {error}"))
        .and_then(|()| {
            window
                .set_focus()
                .map_err(|error| format!("Could not focus input: {error}"))
        });
    app.emit_to("main", "selection-input", event)
        .map_err(|error| format!("Could not deliver input: {error}"))?;
    focus
}

pub fn finish(
    app: &tauri::AppHandle,
    request: u64,
    result: Result<String, SelectionError>,
) -> Result<(), String> {
    let event = match result {
        Ok(text) => SelectionEvent::Captured { request, text },
        Err(error) => SelectionEvent::Failed {
            request,
            code: error.code,
            message: error.message,
        },
    };
    deliver(app, event)
}

pub fn start(app: &tauri::AppHandle, text: Option<String>) -> Result<(), String> {
    let state = app.state::<SelectionState>();
    let request = begin(app)?;
    if let Some(text) = text {
        return finish(app, request, validate(text));
    }
    if state.busy.swap(true, Ordering::SeqCst) {
        return finish(app, request, Err(SelectionError::new("busy", "")));
    }
    #[cfg(any(windows, target_os = "macos"))]
    let source = match platform::Source::current() {
        Ok(source) => source,
        Err(error) => {
            state.busy.store(false, Ordering::SeqCst);
            return finish(app, request, Err(error));
        }
    };
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        state.busy.store(false, Ordering::SeqCst);
        finish(
            app,
            request,
            Err(SelectionError::new(
                "unsupported",
                "Selection capture requires Windows or macOS",
            )),
        )
    }
    #[cfg(any(windows, target_os = "macos"))]
    {
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker_app = app.clone();
        std::thread::spawn(move || {
            let result = source.read().and_then(validate);
            worker_app
                .state::<SelectionState>()
                .busy
                .store(false, Ordering::SeqCst);
            let _ = sender.send(result);
        });
        let waiter_app = app.clone();
        std::thread::spawn(move || {
            let result = receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap_or_else(|error| {
                    Err(SelectionError::new(
                        "read_failed",
                        format!("Selection provider did not finish: {error}"),
                    ))
                });
            if let Err(error) = finish(&waiter_app, request, result) {
                eprintln!("{error}");
            }
        });
        Ok(())
    }
}

#[tauri::command]
pub fn selection_capture(app: tauri::AppHandle, text: Option<String>) -> Result<(), String> {
    start(&app, text)
}

#[tauri::command]
pub fn selection_cancel(app: tauri::AppHandle) -> Result<(), String> {
    app.state::<SelectionState>()
        .sequence
        .fetch_add(1, Ordering::SeqCst);
    crate::ocr_capture::clear(&app)
}

pub fn invalidate_request(app: &tauri::AppHandle, request: u64) {
    let _ = app.state::<SelectionState>().sequence.compare_exchange(
        request,
        request + 1,
        Ordering::SeqCst,
        Ordering::SeqCst,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_preserves_text_and_truncates_to_five_thousand_characters() {
        assert_eq!(
            validate("  école d’aujourd’hui  \r\n".into()).unwrap(),
            "  école d’aujourd’hui  \r\n"
        );
        assert_eq!(validate(" \r\n".into()).unwrap_err().code, "no_selection");
        assert_eq!(
            validate("é".repeat(MAX_TEXT)).unwrap(),
            "é".repeat(MAX_TEXT)
        );
        assert_eq!(
            validate("é".repeat(MAX_TEXT + 1)).unwrap(),
            "é".repeat(MAX_TEXT)
        );
        assert_eq!(
            validate(format!("{}😀fin", "é".repeat(MAX_TEXT - 1))).unwrap(),
            format!("{}😀", "é".repeat(MAX_TEXT - 1))
        );
    }
}
