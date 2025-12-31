// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod storage;

#[cfg(test)]
mod storage_tests;

use serde::Serialize;

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

#[tauri::command]
fn get_active_file() -> storage::ActiveFile {
    let root = storage::storage_root();
    storage::get_active_file_for_date(&root, &today_string())
}

#[tauri::command]
fn save_active_file(filename: String, text: String) -> storage::Counts {
    let root = storage::storage_root();
    storage::save_active_file(&root, &filename, &text)
}

#[tauri::command]
fn archive_item(filename: String, line_idx: usize) -> ArchiveResult {
    let root = storage::storage_root();
    let text = storage::load_or_create(&root, &filename);
    let updated = storage::archive_line(&text, line_idx);
    let counts = storage::save_active_file(&root, &filename, &updated);

    ArchiveResult {
        text: updated,
        counts,
    }
}

#[tauri::command]
fn list_files() -> FileList {
    let root = storage::storage_root();
    let active = storage::get_active_file_for_date(&root, &today_string());
    let files = storage::list_markdown_files(&root);

    FileList {
        files,
        counts: active.counts,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
