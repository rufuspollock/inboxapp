// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod storage;

#[cfg(test)]
mod storage_tests;

use serde::Serialize;
use tauri::{Manager, Wry, tray::TrayIconEvent};

#[derive(Serialize)]
struct ArchiveResult {
    text: String,
    counts: storage::Counts,
}

#[derive(Serialize)]
struct FileList {
    files: Vec<String>,
    counts: storage::Counts,
}

fn today_string() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn format_tray_title(count: usize) -> String {
    let _ = count;
    String::new()
}

fn format_window_title(count: usize) -> String {
    format!("Inbox — {}", count)
}

#[cfg(test)]
mod tray_title_tests {
    use super::format_tray_title;
    use super::format_window_title;

    #[test]
    fn formats_count_in_box() {
        assert_eq!(format_tray_title(7), "");
    }

    #[test]
    fn formats_window_title_with_count() {
        assert_eq!(format_window_title(7), "Inbox — 7");
    }

}

fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn set_tray_title(app: &tauri::AppHandle, count: usize) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(format_tray_title(count)));
    }
}

fn set_window_title(app: &tauri::AppHandle, count: usize) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(&format_window_title(count));
    }
}

#[tauri::command]
fn get_active_file(app: tauri::AppHandle) -> storage::ActiveFile {
    let root = storage::storage_root();
    let active = storage::get_active_file_for_date(&root, &today_string());
    set_tray_title(&app, active.counts.current);
    set_window_title(&app, active.counts.current);
    active
}

#[tauri::command]
fn save_active_file(app: tauri::AppHandle, filename: String, text: String) -> storage::Counts {
    let root = storage::storage_root();
    let counts = storage::save_active_file(&root, &filename, &text);
    set_tray_title(&app, counts.current);
    set_window_title(&app, counts.current);
    counts
}

#[tauri::command]
fn archive_item(app: tauri::AppHandle, filename: String, line_idx: usize) -> ArchiveResult {
    let root = storage::storage_root();
    let text = storage::load_or_create(&root, &filename);
    let updated = storage::archive_line(&text, line_idx);
    let counts = storage::save_active_file(&root, &filename, &updated);
    set_tray_title(&app, counts.current);
    set_window_title(&app, counts.current);

    ArchiveResult {
        text: updated,
        counts,
    }
}

#[tauri::command]
fn list_files(app: tauri::AppHandle) -> FileList {
    let root = storage::storage_root();
    let active = storage::get_active_file_for_date(&root, &today_string());
    let files = storage::list_markdown_files(&root);
    set_tray_title(&app, active.counts.current);
    set_window_title(&app, active.counts.current);

    FileList {
        files,
        counts: active.counts,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(icon) = app.default_window_icon().cloned() {
                let root = storage::storage_root();
                let active = storage::get_active_file_for_date(&root, &today_string());
                tauri::tray::TrayIconBuilder::<Wry>::with_id("main")
                    .icon(icon)
                    .title(format_tray_title(active.counts.current))
                    .on_tray_icon_event(|tray: &tauri::tray::TrayIcon<Wry>, event| {
                        if matches!(event, TrayIconEvent::Click { .. }) {
                            toggle_main_window(&tray.app_handle());
                        }
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_active_file,
            save_active_file,
            archive_item,
            list_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
