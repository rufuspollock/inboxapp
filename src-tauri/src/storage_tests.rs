use crate::storage::{
    archive_line, count_items, get_active_file_for_date, journal_filename, load_or_create,
    save_active_file,
};
use tempfile::tempdir;

#[test]
fn journal_filename_for_today() {
    let date = "2025-12-31";
    assert_eq!(journal_filename(date), "2025-12-31.md");
}

#[test]
fn archive_moves_line_under_archived() {
    let input = "- one\n- two\n\n## Archived\n- old\n";
    let output = archive_line(input, 0);
    assert!(output.contains("- two"));
    assert!(output.contains("## Archived"));
    assert!(output.contains("- one"));
}

#[test]
fn load_creates_file_if_missing() {
    let root = tempdir().unwrap();
    let content = load_or_create(root.path(), "2025-12-31.md");
    assert!(content.is_empty());
    assert!(root.path().join("2025-12-31.md").exists());
}

#[test]
fn count_items_ignores_archived_section() {
    let text = "- one\n- two\n\n## Archived\n- old\n";
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
fn save_active_file_updates_counts() {
    let root = tempdir().unwrap();
    let counts = save_active_file(root.path(), "2025-12-31.md", "- one\n- two\n");
    assert_eq!(counts.current, 2);
    assert_eq!(counts.total, 2);
    assert_eq!(counts.files, 1);
}
