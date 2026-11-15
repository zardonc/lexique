#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "windows")]
    {
        if let Ok(executable) = std::env::current_exe()
            && let Some(parent) = executable.parent()
        {
            let data = parent.join("Lexique-data");
            let temporary = data.join("temp");
            if let Err(error) = std::fs::create_dir_all(&temporary) {
                eprintln!(
                    "Could not prepare portable temp {}: {error}",
                    temporary.display()
                );
            }
            // SAFETY: No application threads exist before Tauri starts. Startup reports an
            // unusable intended temp path before opening learning data; the shell may still run.
            unsafe {
                std::env::set_var("TEMP", &temporary);
                std::env::set_var("TMP", &temporary);
                std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", data.join("webview/EBWebView"));
            }
        } else {
            eprintln!(
                "Could not locate the portable executable; startup must resolve its data directory"
            );
        }
    }
    lexique_desktop_lib::run();
}
