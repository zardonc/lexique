#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "windows")]
    {
        let executable = std::env::current_exe().expect("Cannot locate the portable executable");
        let data = executable
            .parent()
            .expect("The executable must have a parent directory")
            .join("Lexique-data");
        let temporary = data.join("temp");
        std::fs::create_dir_all(&temporary)
            .expect("Cannot create Lexique-data/temp beside the executable; use a writable folder");
        // SAFETY: No application threads exist before Tauri starts.
        unsafe {
            std::env::set_var("TEMP", &temporary);
            std::env::set_var("TMP", &temporary);
            std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", data.join("webview/EBWebView"));
        }
    }
    lexique_desktop_lib::run();
}
