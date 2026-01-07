use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const ITEM_DIVIDER: &str = "---";
const TRASH_FILENAME: &str = "trash.md";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Counts {
    pub current: usize,
    pub total: usize,
    pub files: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DayCount {
    pub date: String,
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DayItems {
    pub date: String,
    pub items: Vec<String>,
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveFile {
    pub filename: String,
    pub text: String,
    pub counts: Counts,
}

fn counts_for_root(root: &Path, active_filename: &str, active_text: &str) -> Counts {
    let mut files = list_markdown_files(root);
    if !files.iter().any(|name| name == active_filename) {
        files.push(active_filename.to_string());
        files.sort();
    }

    let mut total = 0;
    for name in &files {
        if name == active_filename {
            total += count_items(active_text);
            continue;
        }
        let path = root.join(name);
        if let Ok(text) = std::fs::read_to_string(&path) {
            total += count_items(&text);
        }
    }

    Counts {
        current: count_items(active_text),
        total,
        files: files.len(),
    }
}
pub fn storage_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".inboxapp")
}

pub fn journal_filename(date: &str) -> String {
    format!("{date}.md")
}

pub fn split_items(text: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in text.lines() {
        if current.is_empty() && line.trim().is_empty() {
            continue;
        }
        if line.trim() == ITEM_DIVIDER {
            let item = current.join("\n").trim_end().to_string();
            if !item.trim().is_empty() {
                items.push(item);
            }
            current.clear();
            continue;
        }
        current.push(line);
    }

    let item = current.join("\n").trim_end().to_string();
    if !item.trim().is_empty() {
        items.push(item);
    }

    items
}

pub fn append_item_to_text(existing: &str, item: &str) -> String {
    let item = item.trim_end();
    if item.trim().is_empty() {
        return existing.to_string();
    }

    let mut out = existing.trim_end().to_string();
    if !out.is_empty() {
        out.push_str("\n\n---\n\n");
    }
    out.push_str(item);
    out.push('\n');
    out
}

pub fn count_items(text: &str) -> usize {
    split_items(text).len()
}

pub fn read_items_for_date(root: &Path, date: &str) -> Vec<String> {
    let filename = journal_filename(date);
    let path = root.join(filename);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    if text.trim().is_empty() {
        return Vec::new();
    }
    split_items(&text)
}

fn items_to_text(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    format!("{}\n", items.join("\n\n---\n\n"))
}

pub fn list_markdown_files(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == TRASH_FILENAME {
                    continue;
                }
                files.push(name.to_string());
            }
        }
    }
    files.sort();
    files
}

pub fn list_day_counts(root: &Path) -> Vec<DayCount> {
    let mut out = Vec::new();
    for name in list_markdown_files(root) {
        let date = name.trim_end_matches(".md").to_string();
        let path = root.join(&name);
        let text = std::fs::read_to_string(path).unwrap_or_default();
        out.push(DayCount {
            date,
            count: count_items(&text),
        });
    }
    out
}

pub fn load_or_create(root: &Path, filename: &str) -> String {
    let path = root.join(filename);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if !path.exists() {
        let _ = std::fs::write(&path, "");
        return String::new();
    }
    std::fs::read_to_string(&path).unwrap_or_default()
}

pub fn get_active_file_for_date(root: &Path, date: &str) -> ActiveFile {
    let _ = std::fs::create_dir_all(root);
    let filename = journal_filename(date);
    let text = load_or_create(root, &filename);
    let counts = counts_for_root(root, &filename, &text);

    ActiveFile {
        filename,
        text,
        counts,
    }
}

pub fn save_active_file(root: &Path, filename: &str, text: &str) -> Counts {
    let _ = std::fs::create_dir_all(root);
    let path = root.join(filename);
    let _ = std::fs::write(&path, text);
    counts_for_root(root, filename, text)
}

pub fn append_item(root: &Path, filename: &str, item: &str) -> Counts {
    let text = load_or_create(root, filename);
    let updated = append_item_to_text(&text, item);
    save_active_file(root, filename, &updated)
}

pub fn append_item_for_date(root: &Path, date: &str, item: &str) -> Counts {
    let filename = journal_filename(date);
    append_item(root, &filename, item)
}

pub fn update_item_for_date(root: &Path, date: &str, index: usize, item: &str) -> DayItems {
    let filename = journal_filename(date);
    let text = load_or_create(root, &filename);
    let mut items = split_items(&text);
    if index < items.len() {
        items[index] = item.to_string();
    }
    let updated = items_to_text(&items);
    save_active_file(root, &filename, &updated);
    DayItems {
        date: date.to_string(),
        count: items.len(),
        items,
    }
}

pub fn delete_item_for_date(root: &Path, date: &str, index: usize) -> DayItems {
    let filename = journal_filename(date);
    let text = load_or_create(root, &filename);
    let mut items = split_items(&text);
    if index < items.len() {
        let removed = items.remove(index);
        let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();
        let trash_entry = format!("[{}]\n{}", timestamp, removed);
        let trash_path = root.join(TRASH_FILENAME);
        let trash_text = std::fs::read_to_string(&trash_path).unwrap_or_default();
        let trash_updated = append_item_to_text(&trash_text, &trash_entry);
        let _ = std::fs::write(trash_path, trash_updated);
    }
    let updated = items_to_text(&items);
    save_active_file(root, &filename, &updated);
    DayItems {
        date: date.to_string(),
        count: items.len(),
        items,
    }
}
