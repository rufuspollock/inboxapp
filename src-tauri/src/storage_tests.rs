use crate::storage::*;
use std::fs;
use tempfile::tempdir;

const DATE: &str = "2026-10-25";

#[test]
fn migrates_daily_files_once_with_backups_and_preserves_order() {
    let root = tempdir().unwrap();
    let legacy = "first\n\n---\n\n- [x] second\n\nparagraph\n";
    fs::write(root.path().join("2026-10-25.md"), legacy).unwrap();
    fs::write(root.path().join("trash.md"), "[2026-10-24 12:00]\nold\n").unwrap();
    initialize(root.path()).unwrap();
    let items = read_items_for_date(root.path(), DATE).unwrap();
    assert_eq!(
        items.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(),
        vec!["first", "- [x] second\n\nparagraph"]
    );
    assert_ne!(items[0].id, items[1].id);
    assert_eq!(
        fs::read_to_string(root.path().join("legacy-backup-v1/2026-10-25.md")).unwrap(),
        legacy
    );
    assert!(!root.path().join("2026-10-25.md").exists());
    assert!(root.path().join("legacy-backup-v1/trash.md").exists());
    assert!(fs::read_to_string(root.path().join("config.yml"))
        .unwrap()
        .contains("schema_version: 2"));
    initialize(root.path()).unwrap();
    assert_eq!(read_items_for_date(root.path(), DATE).unwrap(), items);
}

#[test]
fn separate_entries_preserve_markdown_dividers_and_stable_ids() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let first = append_entry(root.path(), DATE, "one\n\n---\n\ntwo", None).unwrap();
    let second = append_entry(root.path(), DATE, "Other", None).unwrap();
    assert_eq!(read_items_for_date(root.path(), DATE).unwrap().len(), 2);
    assert!(root.path().join(format!("{}.md", first.id)).exists());
    update_entry(root.path(), DATE, &first.id, "Changed", "one\n\n---\n\ntwo").unwrap();
    assert_eq!(read_items_for_date(root.path(), DATE).unwrap()[1], second);
    assert!(update_entry(root.path(), DATE, &first.id, "bad", "stale").is_err());
}

#[test]
fn reserved_ids_are_unique_and_saves_are_retry_safe() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let a = reserve_entry_id(root.path(), DATE).unwrap();
    let b = reserve_entry_id(root.path(), DATE).unwrap();
    assert_ne!(a, b);
    let entry = append_entry(root.path(), DATE, "one", Some(&a)).unwrap();
    assert_eq!(
        append_entry(root.path(), DATE, "one", Some(&a)).unwrap(),
        entry
    );
    assert!(append_entry(root.path(), DATE, "different", Some(&a)).is_err());
    discard_draft(root.path(), &a).unwrap_err();
    assert!(root.path().join(format!("{a}.md")).exists());
}

#[test]
fn delete_moves_only_selected_entry_and_assets_to_recoverable_bundle() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let a = append_entry(root.path(), DATE, "first", None).unwrap();
    let b = append_entry(root.path(), DATE, "second", None).unwrap();
    let assets = root.path().join("assets").join(&a.id);
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("image.png"), "example").unwrap();
    assert!(delete_entry(root.path(), DATE, &a.id, "stale").is_err());
    delete_entry(root.path(), DATE, &a.id, "first").unwrap();
    assert_eq!(read_items_for_date(root.path(), DATE).unwrap(), vec![b]);
    let trash = root.path().join(".trash").join(&a.id);
    assert_eq!(
        fs::read_to_string(trash.join(format!("{}.md", a.id))).unwrap(),
        "first\n"
    );
    assert!(trash.join("assets").join(&a.id).join("image.png").exists());
    assert!(!assets.exists());
}

#[test]
fn migration_failure_is_resumable_without_duplicate_entries_or_data_loss() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("2026-10-25.md"), "one\n\n---\n\ntwo\n").unwrap();
    fs::create_dir(root.path().join("2026-10-25-0002.md")).unwrap();
    assert!(initialize(root.path()).is_err());
    assert!(root.path().join("2026-10-25.md").exists());
    assert!(!root.path().join("config.yml").exists());
    fs::remove_dir(root.path().join("2026-10-25-0002.md")).unwrap();
    initialize(root.path()).unwrap();
    assert_eq!(read_items_for_date(root.path(), DATE).unwrap().len(), 2);
}

#[test]
fn rejects_future_schema_bad_dates_paths_and_write_failures() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("config.yml"), "schema_version: 999\n").unwrap();
    assert!(initialize(root.path()).is_err());
    assert!(read_items_for_date(root.path(), "../../escape").is_err());
    assert!(append_entry(root.path(), "2026-02-30", "bad", None).is_err());
    let file = root.path().join("not-directory");
    fs::write(&file, "keep").unwrap();
    assert!(initialize(&file).is_err());
}

#[test]
fn counts_only_active_entries_and_keeps_empty_days_out() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    assert!(read_items_for_date(root.path(), DATE).unwrap().is_empty());
    assert!(!root.path().join("2026-10-25.md").exists());
    let entry = append_entry(root.path(), DATE, "one", None).unwrap();
    append_entry(root.path(), "2026-10-24", "old", None).unwrap();
    assert_eq!(list_day_counts(root.path()).unwrap().len(), 2);
    delete_entry(root.path(), DATE, &entry.id, "one").unwrap();
    assert_eq!(
        list_day_counts(root.path()).unwrap(),
        vec![DayCount {
            date: "2026-10-24".into(),
            count: 1
        }]
    );
}

#[test]
fn migration_refuses_changed_original_after_interruption() {
    let root = tempdir().unwrap();
    let source = root.path().join("2026-10-25.md");
    fs::write(&source, "one\n\n---\n\ntwo\n").unwrap();
    fs::create_dir(root.path().join("2026-10-25-0002.md")).unwrap();
    initialize(root.path()).unwrap_err();
    fs::write(&source, "changed externally").unwrap();
    fs::remove_dir(root.path().join("2026-10-25-0002.md")).unwrap();
    assert!(initialize(root.path())
        .unwrap_err()
        .to_string()
        .contains("changed during upgrade"));
    assert_eq!(fs::read_to_string(source).unwrap(), "changed externally");
    assert!(!root.path().join("config.yml").exists());
}

#[test]
fn deleting_with_unwritable_trash_leaves_original_and_assets_intact() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let entry = append_entry(root.path(), DATE, "keep", None).unwrap();
    fs::write(root.path().join(".trash"), "not directory").unwrap();
    assert!(delete_entry(root.path(), DATE, &entry.id, "keep").is_err());
    assert_eq!(read_items_for_date(root.path(), DATE).unwrap(), vec![entry]);
}

#[test]
fn restart_finishes_interrupted_trash_bundle_and_cleans_owned_assets() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let entry = append_entry(root.path(), DATE, "keep", None).unwrap();
    let pending = root
        .path()
        .join(".trash")
        .join(format!(".pending-{}", entry.id));
    let source_assets = root.path().join("assets").join(&entry.id);
    let copied_assets = pending.join("assets").join(&entry.id);
    fs::create_dir_all(&source_assets).unwrap();
    fs::create_dir_all(&copied_assets).unwrap();
    fs::write(source_assets.join("image.png"), "image").unwrap();
    fs::write(copied_assets.join("image.png"), "image").unwrap();
    fs::rename(
        root.path().join(format!("{}.md", entry.id)),
        pending.join(format!("{}.md", entry.id)),
    )
    .unwrap();
    initialize(root.path()).unwrap();
    assert!(root
        .path()
        .join(".trash")
        .join(&entry.id)
        .join("assets")
        .join(&entry.id)
        .join("image.png")
        .exists());
    assert!(!source_assets.exists());
    assert!(read_items_for_date(root.path(), DATE).unwrap().is_empty());
    initialize(root.path()).unwrap();
}

#[test]
fn new_ids_are_not_reused_after_permanent_removal() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let a = reserve_entry_id(root.path(), DATE).unwrap();
    discard_draft(root.path(), &a).unwrap();
    let b = reserve_entry_id(root.path(), DATE).unwrap();
    assert_ne!(a, b);
}

#[test]
fn bad_config_is_reported_without_touching_legacy_files() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("config.yml"), "schema_version: [oops").unwrap();
    fs::write(root.path().join("2026-10-25.md"), "keep").unwrap();
    assert!(initialize(root.path()).is_err());
    assert_eq!(
        fs::read_to_string(root.path().join("2026-10-25.md")).unwrap(),
        "keep"
    );
    assert!(!root.path().join("legacy-backup-v1").exists());
}

#[test]
fn migration_persists_id_counter_before_entries_are_processed_elsewhere() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("2026-10-25.md"), "one\n\n---\n\ntwo\n").unwrap();
    initialize(root.path()).unwrap();
    let entries = read_items_for_date(root.path(), DATE).unwrap();
    for entry in &entries { fs::remove_file(root.path().join(format!("{}.md", entry.id))).unwrap(); }
    assert_eq!(reserve_entry_id(root.path(), DATE).unwrap(), "2026-10-25-0003");
}

#[test]
fn unreadable_other_entry_prevents_mutation_before_any_side_effect() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let entry = append_entry(root.path(), DATE, "original", None).unwrap();
    fs::write(root.path().join("2026-10-25-0002.md"), [0xff, 0xfe]).unwrap();
    assert!(update_entry(root.path(), DATE, &entry.id, "changed", "original").is_err());
    assert!(delete_entry(root.path(), DATE, &entry.id, "original").is_err());
    assert_eq!(fs::read_to_string(root.path().join(format!("{}.md", entry.id))).unwrap(), "original\n");
    assert!(!root.path().join(".trash").exists());
}
