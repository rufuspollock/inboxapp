// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod clipboard;
mod storage;

#[cfg(test)]
mod storage_tests;

use serde::Serialize;
use tauri::{image::Image, menu::Menu, tray::TrayIconEvent, Manager, Wry};
use tauri_plugin_opener::OpenerExt;

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

#[derive(Debug, Serialize)]
struct PasteResult {
    attachments: Vec<storage::Attachment>,
    text: String,
    error: Option<String>,
}

fn import_contents(
    root: &std::path::Path,
    id: &str,
    contents: clipboard::Contents,
) -> std::io::Result<PasteResult> {
    let mut result = PasteResult {
        attachments: Vec::new(),
        text: String::new(),
        error: None,
    };
    let image_bytes = match contents {
        clipboard::Contents::Files(files) => {
            let mut errors = Vec::new();
            for source in files {
                match storage::import_file_path(root, id, &source) {
                    Ok(attachment) => result.attachments.push(attachment),
                    Err(error) => errors.push(format!(
                        "{}: {error}",
                        source.file_name().unwrap_or_default().to_string_lossy()
                    )),
                }
            }
            if !errors.is_empty() {
                result.error = Some(errors.join("; "));
            }
            return Ok(result);
        }
        clipboard::Contents::Text(text) => {
            result.text = text;
            return Ok(result);
        }
        clipboard::Contents::Empty => return Ok(result),
        clipboard::Contents::Png(bytes) => bytes,
        clipboard::Contents::Image {
            width,
            height,
            rgba,
        } => storage::encode_clipboard_image(width, height, rgba)?,
    };
    let path = storage::import_attachment(root, id, &image_bytes)?;
    result.attachments.push(storage::Attachment {
        path,
        name: "Image".into(),
        is_image: true,
    });
    Ok(result)
}

#[cfg(test)]
mod paste_tests {
    use super::*;

    #[test]
    fn file_batch_keeps_successes_and_reports_failures_without_touching_sources() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        storage::initialize(root.path()).unwrap();
        let id = storage::reserve_entry_id(root.path(), "2026-10-26").unwrap();
        let pdf = source.path().join("document.pdf");
        let missing = source.path().join("missing.file");
        let binary = source.path().join("unknown.asset");
        std::fs::write(&pdf, b"%PDF-test").unwrap();
        std::fs::write(&binary, [0, 255, 1]).unwrap();
        let result = import_contents(
            root.path(),
            &id,
            clipboard::Contents::Files(vec![pdf.clone(), missing, binary.clone()]),
        )
        .unwrap();
        assert_eq!(result.attachments.len(), 2);
        assert!(result.error.unwrap().contains("missing.file"));
        assert!(result.text.is_empty());
        for (attachment, original) in result.attachments.iter().zip([pdf, binary]) {
            assert!(!attachment.is_image);
            assert_eq!(
                std::fs::read(root.path().join(&attachment.path)).unwrap(),
                std::fs::read(original).unwrap()
            );
        }
    }

    #[test]
    fn pixel_and_png_clipboards_produce_durable_validated_image_attachments() {
        let root = tempfile::tempdir().unwrap();
        storage::initialize(root.path()).unwrap();
        let id = storage::reserve_entry_id(root.path(), "2026-10-26").unwrap();
        let png = storage::encode_clipboard_image(1, 1, vec![255, 0, 0, 255]).unwrap();
        for contents in [
            clipboard::Contents::Image {
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 255],
            },
            clipboard::Contents::Png(png),
        ] {
            let result = import_contents(root.path(), &id, contents).unwrap();
            assert_eq!(result.attachments.len(), 1);
            assert!(result.attachments[0].is_image);
            assert!(
                storage::read_attachment(root.path(), &id, &result.attachments[0].path)
                    .unwrap()
                    .starts_with("data:image/png;base64,")
            );
            assert!(result.error.is_none());
        }
        assert!(import_contents(
            root.path(),
            &id,
            clipboard::Contents::Png(b"not a PNG".to_vec())
        )
        .is_err());
    }

    #[test]
    fn text_and_empty_clipboards_do_not_create_assets() {
        let root = tempfile::tempdir().unwrap();
        let result = import_contents(
            root.path(),
            "unused",
            clipboard::Contents::Text("selection replacement".into()),
        )
        .unwrap();
        assert_eq!(result.text, "selection replacement");
        assert!(result.attachments.is_empty());
        let empty = import_contents(root.path(), "unused", clipboard::Contents::Empty).unwrap();
        assert!(empty.attachments.is_empty());
        assert!(empty.text.is_empty());
        assert!(empty.error.is_none());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
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
async fn import_file(
    id: String,
    name: String,
    bytes: Vec<u8>,
) -> Result<storage::Attachment, String> {
    tauri::async_runtime::spawn_blocking(move || {
        with_storage(|root| storage::import_file(root, &id, &name, &bytes))
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
async fn paste_attachments(id: String) -> Result<PasteResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let contents = clipboard::read()?;
        with_storage(|root| import_contents(root, &id, contents))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn open_attachment(app: tauri::AppHandle, id: String, path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let owned = with_storage(|root| storage::resolve_attachment(root, &id, &path))?;
        app.opener()
            .open_path(owned.to_string_lossy().into_owned(), None::<&str>)
            .map_err(|error| format!("Cannot open attachment: {error}"))
    })
    .await
    .map_err(|e| e.to_string())?
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
            import_file,
            read_attachment,
            paste_attachments,
            open_attachment,
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
