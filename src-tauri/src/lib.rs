// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod storage;

#[cfg(test)]
mod storage_tests;

use serde::Serialize;
use tauri::{image::Image, menu::Menu, tray::TrayIconEvent, Manager, Wry};
use tauri_plugin_clipboard_manager::ClipboardExt;

static STORAGE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Serialize)]
struct TodayItems {
    date: String,
    filename: String,
    items: Vec<storage::Entry>,
    counts: storage::Counts,
}
#[derive(Serialize)]
struct SavedEntry {
    entry: storage::Entry,
    date: String,
    counts: storage::Counts,
}

fn with_storage<T>(
    operation: impl FnOnce(&std::path::Path) -> std::io::Result<T>,
) -> Result<T, String> {
    let _guard = STORAGE_LOCK
        .lock()
        .map_err(|_| "Storage lock unavailable".to_string())?;
    let root = storage::storage_root();
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(root.join(".storage.lock"))
        .map_err(|e| e.to_string())?;
    lock.lock().map_err(|e| e.to_string())?;
    storage::initialize(&root).map_err(|e| e.to_string())?;
    operation(&root).map_err(|e| e.to_string())
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
fn get_today_items(app: tauri::AppHandle) -> Result<TodayItems, String> {
    let date = today_string();
    let data = with_storage(|root| {
        Ok(TodayItems {
            filename: format!("{date}.md"),
            items: storage::read_items_for_date(root, &date)?,
            counts: storage::counts(root, &date)?,
            date: date.clone(),
        })
    })?;
    set_tray_title(&app, data.counts.current);
    set_window_title(&app, data.counts.current);
    Ok(data)
}

#[tauri::command]
fn get_items_for_date(app: tauri::AppHandle, date: String) -> Result<storage::DayItems, String> {
    let data = with_storage(|root| storage::day_items(root, &date))?;
    set_window_title(&app, data.count);
    Ok(data)
}

#[tauri::command]
fn list_day_counts() -> Result<Vec<storage::DayCount>, String> {
    with_storage(storage::list_day_counts)
}

#[tauri::command]
fn new_entry_id() -> Result<String, String> {
    with_storage(|root| storage::reserve_entry_id(root, &today_string()))
}

#[tauri::command]
fn discard_draft(id: String) -> Result<(), String> {
    with_storage(|root| storage::discard_draft(root, &id))
}

#[tauri::command]
fn append_today_item(
    app: tauri::AppHandle,
    text: String,
    id: Option<String>,
) -> Result<SavedEntry, String> {
    let date = match id.as_deref() {
        Some(id) => storage::id_date(id).map_err(|e| e.to_string())?.to_string(),
        None => today_string(),
    };
    let result = with_storage(|root| {
        Ok(SavedEntry {
            entry: storage::append_entry(root, &date, &text, id.as_deref())?,
            counts: storage::counts(root, &date)?,
            date: date.clone(),
        })
    })?;
    if date == today_string() {
        set_tray_title(&app, result.counts.current);
    }
    set_window_title(&app, result.counts.current);
    Ok(result)
}

#[tauri::command]
fn update_item_for_date(
    app: tauri::AppHandle,
    date: String,
    id: String,
    item: String,
    expected_item: String,
) -> Result<storage::DayItems, String> {
    let updated =
        with_storage(|root| storage::update_entry(root, &date, &id, &item, &expected_item))?;
    if date == today_string() {
        set_tray_title(&app, updated.count);
    }
    set_window_title(&app, updated.count);
    Ok(updated)
}

#[tauri::command]
fn delete_item_for_date(
    app: tauri::AppHandle,
    date: String,
    id: String,
    expected_item: String,
) -> Result<storage::DayItems, String> {
    let updated = with_storage(|root| storage::delete_entry(root, &date, &id, &expected_item))?;
    if date == today_string() {
        set_tray_title(&app, updated.count);
    }
    set_window_title(&app, updated.count);
    Ok(updated)
}

#[tauri::command]
async fn import_attachment(id: String, bytes: Vec<u8>) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        with_storage(|root| storage::import_attachment(root, &id, &bytes))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn read_attachment(id: String, path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        with_storage(|root| storage::read_attachment(root, &id, &path))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn paste_image(app: tauri::AppHandle, id: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let image = match app.clipboard().read_image() {
            Ok(image) => image,
            Err(error) if error.to_string() == "The clipboard contents were not available in the requested format or the clipboard is empty." => return Ok(None),
            Err(error) => return Err(format!("Cannot read clipboard image: {error}")),
        };
        let bytes = storage::encode_clipboard_image(image.width(), image.height(), image.rgba().to_vec()).map_err(|e| e.to_string())?;
        with_storage(|root| storage::import_attachment(root, &id, &bytes)).map(Some)
    }).await.map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(icon) = app.default_window_icon().cloned() {
                // Initialize through the UI commands so migration errors are visible in the window.
                let count = 0;
                let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                    .map(|image| image.to_owned())
                    .unwrap_or_else(|_| icon.clone());
                let mut tray_builder = tauri::tray::TrayIconBuilder::<Wry>::with_id("main")
                    .icon(tray_icon)
                    .title(format_tray_title(count))
                    .on_tray_icon_event(|tray: &tauri::tray::TrayIcon<Wry>, event| {
                        if matches!(event, TrayIconEvent::Click { .. }) {
                            toggle_main_window(tray.app_handle());
                        }
                    });
                #[cfg(target_os = "macos")]
                {
                    tray_builder = tray_builder.icon_as_template(true);
                }
                tray_builder.build(app)?;
            }
            Ok(())
        })
        .menu(Menu::default)
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            import_attachment,
            read_attachment,
            paste_image,
            new_entry_id,
            discard_draft,
            get_today_items,
            append_today_item,
            get_items_for_date,
            list_day_counts,
            update_item_for_date,
            delete_item_for_date
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
