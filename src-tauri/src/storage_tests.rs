use crate::storage::{
    append_item_for_date, append_item_to_text, count_items, get_active_file_for_date,
    journal_filename, load_or_create, read_items_for_date, save_active_file, split_items,
};
use tempfile::tempdir;

#[test]
fn journal_filename_for_today() {
    let date = "2025-12-31";
    assert_eq!(journal_filename(date), "2025-12-31.md");
}

#[test]
fn load_creates_file_if_missing() {
    let root = tempdir().unwrap();
    let content = load_or_create(root.path(), "2025-12-31.md");
    assert!(content.is_empty());
    assert!(root.path().join("2025-12-31.md").exists());
}

#[test]
fn split_items_uses_markdown_divider() {
    let text = "first\n\n---\n\nsecond\n";
    let items = split_items(text);
    assert_eq!(items, vec!["first", "second"]);
}

#[test]
fn append_item_adds_divider() {
    let first = append_item_to_text("", "first");
    let second = append_item_to_text(&first, "second");
    assert!(second.contains("---"));
    assert!(second.contains("first"));
    assert!(second.contains("second"));
}

#[test]
fn count_items_counts_divided_items() {
    let text = "first\n\n---\n\nsecond\n";
    assert_eq!(count_items(text), 2);
}

#[test]
fn get_active_file_creates_daily_file() {
    let root = tempdir().unwrap();
    let result = get_active_file_for_date(root.path(), "2025-12-31");
    assert_eq!(result.filename, "2025-12-31.md");
    assert!(result.text.is_empty());
    assert_eq!(result.counts.current, 0);
    assert_eq!(result.counts.total, 0);
    assert_eq!(result.counts.files, 1);
}

#[test]
fn read_items_for_date_does_not_create_file() {
    let root = tempdir().unwrap();
    let items = read_items_for_date(root.path(), "2026-01-01");
    let path = root.path().join("2026-01-01.md");

    assert!(items.is_empty());
    assert!(!path.exists());
}

#[test]
fn save_active_file_updates_counts() {
    let root = tempdir().unwrap();
    let counts = save_active_file(root.path(), "2025-12-31.md", "one\n\n---\n\ntwo\n");
    assert_eq!(counts.current, 2);
    assert_eq!(counts.total, 2);
    assert_eq!(counts.files, 1);
}

#[test]
fn append_item_for_date_writes_to_daily_file() {
    let root = tempdir().unwrap();
    let counts = append_item_for_date(root.path(), "2025-12-31", "first");
    let path = root.path().join("2025-12-31.md");

    assert!(path.exists());
    assert_eq!(counts.current, 1);
    assert_eq!(counts.total, 1);
    assert_eq!(counts.files, 1);
    assert!(std::fs::read_to_string(path).unwrap().contains("first"));
}
