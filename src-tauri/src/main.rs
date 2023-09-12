// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn alert_window(window: tauri::Window) {
    window
        .request_user_attention(Some(tauri::UserAttentionType::Critical))
        .expect("should be able to request user attention");
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![alert_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
