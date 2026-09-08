mod commands;
mod engine_resources;
mod progress;
mod tray;

use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::run_optimizer,
            commands::get_default_jobs
        ])
        .on_window_event(|window, event| {
            // Closing the main window hides it to the tray; only the tray's
            // Quit menu item actually exits the app.
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .setup(|app| {
            tray::build(app.handle())?;

            // A login-triggered launch (autostart passes --hidden) stays in
            // the tray; a manual launch shows the window.
            let launched_hidden = std::env::args().any(|arg| arg == "--hidden");
            if !launched_hidden {
                if let Some(window) = app.get_webview_window("main") {
                    window.show()?;
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
