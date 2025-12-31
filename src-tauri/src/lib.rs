// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod storage;

#[cfg(test)]
mod storage_tests;

use serde::Serialize;
use std::path::Path;
use tauri::{image::Image, Manager, Wry, tray::TrayIconEvent};

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

#[cfg(test)]
mod archive_command_tests {
    use super::{archive_item_internal, restore_item_internal};
    use tempfile::tempdir;

    #[test]
    fn archive_item_internal_rejects_text_mismatch() {
        let root = tempdir().unwrap();
        let filename = "2026-01-01.md";
        std::fs::write(root.path().join(filename), "- one\n").unwrap();

        let result = archive_item_internal(root.path(), filename, 0, "- nope");
        assert!(result.is_err());
    }

    #[test]
    fn restore_item_internal_moves_line_to_active() {
        let root = tempdir().unwrap();
        let filename = "2026-01-01.md";
        let contents = "- one\n\n## Archived\n- done\n";
        std::fs::write(root.path().join(filename), contents).unwrap();

        let result = restore_item_internal(root.path(), filename, 0, "- done").unwrap();
        assert!(result.text.contains("- done"));
        assert!(!result.text.contains("## Archived\n- done\n"));
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

fn archive_item_internal(
    root: &Path,
    filename: &str,
    line_idx: usize,
    line_text: &str,
) -> Result<ArchiveResult, storage::ArchiveError> {
    let text = storage::load_or_create(root, filename);
    let updated = storage::archive_line_matching(&text, line_idx, line_text)?;
    let counts = storage::save_active_file(root, filename, &updated);

    Ok(ArchiveResult {
        text: updated,
        counts,
    })
}

fn restore_item_internal(
    root: &Path,
    filename: &str,
    line_idx: usize,
    line_text: &str,
) -> Result<ArchiveResult, storage::ArchiveError> {
    let text = storage::load_or_create(root, filename);
    let updated = storage::restore_line_matching(&text, line_idx, line_text)?;
    let counts = storage::save_active_file(root, filename, &updated);

    Ok(ArchiveResult {
        text: updated,
        counts,
    })
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
fn archive_item(
    app: tauri::AppHandle,
    filename: String,
    line_idx: usize,
    line_text: String,
) -> Result<ArchiveResult, String> {
    let root = storage::storage_root();
    let result = archive_item_internal(&root, &filename, line_idx, &line_text)
        .map_err(|err| format!("archive failed: {:?}", err))?;
    set_tray_title(&app, result.counts.current);
    set_window_title(&app, result.counts.current);
    Ok(result)
}

#[tauri::command]
fn restore_item(
    app: tauri::AppHandle,
    filename: String,
    line_idx: usize,
    line_text: String,
) -> Result<ArchiveResult, String> {
    let root = storage::storage_root();
    let result = restore_item_internal(&root, &filename, line_idx, &line_text)
        .map_err(|err| format!("restore failed: {:?}", err))?;
    set_tray_title(&app, result.counts.current);
    set_window_title(&app, result.counts.current);
    Ok(result)
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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_active_file,
            save_active_file,
            archive_item,
            restore_item,
            list_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
