//! Native failure codes only; never persist page URLs or learning content.
use std::path::PathBuf;

use tauri::Manager;
use webview2_com::{
    Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_PROCESS_FAILED_KIND, COREWEBVIEW2_PROCESS_FAILED_REASON,
        ICoreWebView2ProcessFailedEventArgs2,
    },
    ProcessFailedEventHandler,
};
use windows_core::Interface;

pub fn attach(app: &tauri::App, database: PathBuf) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.with_webview(move |webview| {
            let diagnostic_path = database.clone();
            let handler = ProcessFailedEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
                    let kind_result = unsafe { args.ProcessFailedKind(&mut kind) };
                    let detail = args.cast::<ICoreWebView2ProcessFailedEventArgs2>().and_then(|args| {
                        let mut reason = COREWEBVIEW2_PROCESS_FAILED_REASON::default();
                        let mut exit_code = 0;
                        unsafe { args.Reason(&mut reason)?; args.ExitCode(&mut exit_code)?; }
                        Ok((reason.0, exit_code))
                    });
                    crate::speech_cache::report_error(&diagnostic_path, &format!(
                        "[native-lifecycle] WebView2 ProcessFailed kind={} kind_result={kind_result:?} detail={detail:?}", kind.0
                    ));
                }
                Ok(())
            }));
            let result = unsafe {
                webview.controller().CoreWebView2().and_then(|view| view.add_ProcessFailed(&handler, &mut 0))
            };
            if let Err(error) = result {
                crate::speech_cache::report_error(&database, &format!("[native-lifecycle] WebView2 diagnostics unavailable: {error}"));
            }
        })?;
    }
    Ok(())
}
