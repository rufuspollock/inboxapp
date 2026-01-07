// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod storage;

#[cfg(test)]
mod storage_tests;

use serde::Serialize;
use tauri::{image::Image, menu::Menu, Manager, Wry, tray::TrayIconEvent};

#[derive(Serialize)]
struct FileList {
    files: Vec<String>,
    counts: storage::Counts,
}

#[derive(Serialize)]
struct TodayItems {
    filename: String,
    items: Vec<String>,
    counts: storage::Counts,
}

#[derive(Serialize)]
struct DayItems {
    date: String,
    items: Vec<String>,
    count: usize,
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

#[tauri::command]
fn get_today_items(app: tauri::AppHandle) -> TodayItems {
    let root = storage::storage_root();
    let active = storage::get_active_file_for_date(&root, &today_string());
    let items = storage::split_items(&active.text);
    set_tray_title(&app, active.counts.current);
    set_window_title(&app, active.counts.current);

    TodayItems {
        filename: active.filename,
        items,
        counts: active.counts,
    }
}

#[tauri::command]
fn get_items_for_date(app: tauri::AppHandle, date: String) -> DayItems {
    let root = storage::storage_root();
    let items = storage::read_items_for_date(&root, &date);
    let count = items.len();
    set_window_title(&app, count);

    DayItems { date, items, count }
}

#[tauri::command]
fn list_day_counts(_app: tauri::AppHandle) -> Vec<storage::DayCount> {
    let root = storage::storage_root();
    storage::list_day_counts(&root)
}

#[tauri::command]
fn append_today_item(app: tauri::AppHandle, text: String) -> storage::Counts {
    let root = storage::storage_root();
    let counts = storage::append_item_for_date(&root, &today_string(), &text);
    set_tray_title(&app, counts.current);
    set_window_title(&app, counts.current);
    counts
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(icon) = app.default_window_icon().cloned() {
                let root = storage::storage_root();
                let active = storage::get_active_file_for_date(&root, &today_string());
                let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                    .map(|image| image.to_owned())
                    .unwrap_or_else(|_| icon.clone());
                let mut tray_builder = tauri::tray::TrayIconBuilder::<Wry>::with_id("main")
                    .icon(tray_icon)
                    .title(format_tray_title(active.counts.current))
                    .on_tray_icon_event(|tray: &tauri::tray::TrayIcon<Wry>, event| {
                        if matches!(event, TrayIconEvent::Click { .. }) {
                            toggle_main_window(&tray.app_handle());
                        }
                    })
                    ;
                #[cfg(target_os = "macos")]
                {
                    tray_builder = tray_builder.icon_as_template(true);
                }
                tray_builder.build(app)?;
            }
            Ok(())
        })
        .menu(|app| Menu::default(app))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_active_file,
            save_active_file,
            list_files,
            get_today_items,
            append_today_item,
            get_items_for_date,
            list_day_counts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
