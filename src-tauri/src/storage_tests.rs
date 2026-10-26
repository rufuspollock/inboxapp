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
    for entry in &entries {
        fs::remove_file(root.path().join(format!("{}.md", entry.id))).unwrap();
    }
    assert_eq!(
        reserve_entry_id(root.path(), DATE).unwrap(),
        "2026-10-25-0003"
    );
}

#[test]
fn unreadable_other_entry_prevents_mutation_before_any_side_effect() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let entry = append_entry(root.path(), DATE, "original", None).unwrap();
    fs::write(root.path().join("2026-10-25-0002.md"), [0xff, 0xfe]).unwrap();
    assert!(update_entry(root.path(), DATE, &entry.id, "changed", "original").is_err());
    assert!(delete_entry(root.path(), DATE, &entry.id, "original").is_err());
    assert_eq!(
        fs::read_to_string(root.path().join(format!("{}.md", entry.id))).unwrap(),
        "original\n"
    );
    assert!(!root.path().join(".trash").exists());
}

fn test_png() -> Vec<u8> {
    // One opaque red pixel, encoded using the same public file format as clipboard capture.
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(1, 1, vec![255, 0, 0, 255]).unwrap(),
    )
    .write_to(&mut bytes, image::ImageFormat::Png)
    .unwrap();
    bytes.into_inner()
}

#[test]
fn imports_owned_images_without_overwriting_and_reads_preview() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    let bytes = test_png();
    let first = import_attachment(root.path(), &id, &bytes).unwrap();
    let second = import_attachment(root.path(), &id, &bytes).unwrap();
    assert_ne!(first, second);
    assert_eq!(fs::read(root.path().join(&first)).unwrap(), bytes);
    assert!(read_attachment(root.path(), &id, &first)
        .unwrap()
        .starts_with("data:image/png;base64,"));
    assert!(first.starts_with(&format!("assets/{id}/")));
    discard_draft(root.path(), &id).unwrap();
    assert!(!root.path().join(first).exists());
}

#[test]
fn refuses_invalid_large_and_unowned_images_without_writing() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    assert!(import_attachment(root.path(), &id, b"not an image").is_err());
    assert!(import_attachment(root.path(), &id, &vec![0; 20 * 1024 * 1024 + 1]).is_err());
    assert!(import_attachment(root.path(), "2026-10-25-9999", &test_png()).is_err());
    assert!(!root.path().join("assets").join(&id).exists());
}

#[test]
fn attachment_reads_reject_traversal_and_other_entries() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    let other = reserve_entry_id(root.path(), DATE).unwrap();
    let path = import_attachment(root.path(), &id, &test_png()).unwrap();
    assert!(read_attachment(root.path(), &other, &path).is_err());
    assert!(read_attachment(root.path(), &id, "../../config.yml").is_err());
    assert!(read_attachment(root.path(), &id, &format!("assets/{id}/../config.yml")).is_err());
}

#[cfg(unix)]
#[test]
fn attachment_access_refuses_symlinks_outside_owned_folder() {
    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    fs::create_dir_all(root.path().join("assets")).unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("assets").join(&id)).unwrap();
    assert!(import_attachment(root.path(), &id, &test_png()).is_err());
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
}

#[test]
fn clipboard_pixels_encode_as_png_and_reject_invalid_dimensions() {
    let bytes = encode_clipboard_image(1, 1, vec![255, 0, 0, 255]).unwrap();
    assert_eq!(
        image::guess_format(&bytes).unwrap(),
        image::ImageFormat::Png
    );
    assert!(encode_clipboard_image(1, 2, vec![0; 4]).is_err());
    assert!(encode_clipboard_image(40001, 1000, Vec::new()).is_err());
}

#[test]
fn saved_image_entry_moves_preview_bytes_to_trash_with_its_markdown() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    let path = import_attachment(root.path(), &id, &test_png()).unwrap();
    let text = format!("![Screenshot]({path})");
    append_entry(root.path(), DATE, &text, Some(&id)).unwrap();
    assert!(read_attachment(root.path(), &id, &path).is_ok());
    delete_entry(root.path(), DATE, &id, &text).unwrap();
    assert!(!root.path().join(&path).exists());
    assert!(root.path().join(".trash").join(&id).join(&path).exists());
}

#[test]
fn jpeg_and_webp_imports_preserve_original_bytes() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    for (format, extension) in [
        (image::ImageFormat::Jpeg, "jpg"),
        (image::ImageFormat::WebP, "webp"),
    ] {
        let picture = image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(1, 1, vec![255, 0, 0]).unwrap(),
        );
        let mut buffer = std::io::Cursor::new(Vec::new());
        picture.write_to(&mut buffer, format).unwrap();
        let bytes = buffer.into_inner();
        let path = import_attachment(root.path(), &id, &bytes).unwrap();
        assert!(path.ends_with(extension));
        assert_eq!(fs::read(root.path().join(path)).unwrap(), bytes);
    }
}

#[test]
fn saving_and_editing_reject_another_entries_owned_image_links() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let first = reserve_entry_id(root.path(), DATE).unwrap();
    let path = import_attachment(root.path(), &first, &test_png()).unwrap();
    let text = format!("![Screenshot]({path})");
    append_entry(root.path(), DATE, &text, Some(&first)).unwrap();
    let second = reserve_entry_id(root.path(), DATE).unwrap();
    let error = append_entry(root.path(), DATE, &text, Some(&second)).unwrap_err();
    assert!(error.to_string().contains("belongs to another entry"));
    assert!(!root.path().join(format!("{second}.md")).exists());
    append_entry(root.path(), DATE, "keep", Some(&second)).unwrap();
    assert!(update_entry(root.path(), DATE, &second, &text, "keep").is_err());
    assert_eq!(
        fs::read_to_string(root.path().join(format!("{second}.md"))).unwrap(),
        "keep\n"
    );
}

#[test]
fn arbitrary_files_preserve_bytes_names_and_collisions() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    for (name, bytes) in [
        ("report [final].pdf", b"%PDF-test".as_slice()),
        ("data.unknown", &[0, 255, 1]),
        ("empty", &[]),
    ] {
        let first = import_file(root.path(), &id, name, bytes).unwrap();
        let second = import_file(root.path(), &id, name, bytes).unwrap();
        assert_ne!(first.path, second.path);
        assert_eq!(first.name, name);
        assert!(!first.is_image);
        assert_eq!(
            fs::read(resolve_attachment(root.path(), &id, &first.path).unwrap()).unwrap(),
            bytes
        );
        assert_eq!(fs::read(root.path().join(second.path)).unwrap(), bytes);
        if name.ends_with(".pdf") {
            assert!(first.path.ends_with(".pdf"));
        }
    }
}

#[test]
fn arbitrary_import_limits_and_names_cannot_escape_owned_folder() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    assert!(import_file(root.path(), &id, "big.pdf", &vec![0; 20 * 1024 * 1024 + 1]).is_err());
    assert!(import_file(root.path(), "2026-10-25-9999", "x", b"x").is_err());
    for name in [
        "../../escape.pdf",
        ".",
        "",
        "a\\b\n[x](y).pdf",
        "résumé.pdf",
    ] {
        let file = import_file(root.path(), &id, name, b"contents").unwrap();
        let path = resolve_attachment(root.path(), &id, &file.path).unwrap();
        assert_eq!(path.parent().unwrap(), root.path().join("assets").join(&id));
    }
}

#[test]
fn generic_assets_survive_unlink_and_move_with_entry_or_discard() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    let file = import_file(root.path(), &id, "report.pdf", b"%PDF-test").unwrap();
    let text = format!("[report.pdf]({})", file.path);
    append_entry(root.path(), DATE, &text, Some(&id)).unwrap();
    update_entry(root.path(), DATE, &id, "link removed", &text).unwrap();
    assert!(resolve_attachment(root.path(), &id, &file.path).is_ok());
    delete_entry(root.path(), DATE, &id, "link removed").unwrap();
    assert_eq!(
        fs::read(root.path().join(".trash").join(&id).join(&file.path)).unwrap(),
        b"%PDF-test"
    );
    let draft = reserve_entry_id(root.path(), DATE).unwrap();
    let unsaved = import_file(root.path(), &draft, "data.bin", b"bytes").unwrap();
    discard_draft(root.path(), &draft).unwrap();
    assert!(!root.path().join(unsaved.path).exists());
}

#[cfg(unix)]
#[test]
fn generic_asset_access_refuses_symlinks_traversal_and_other_owners() {
    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    let other = reserve_entry_id(root.path(), DATE).unwrap();
    let file = import_file(root.path(), &id, "with_under_score.pdf", b"%PDF").unwrap();
    assert!(resolve_attachment(root.path(), &other, &file.path).is_err());
    assert!(resolve_attachment(root.path(), &id, "../../config.yml").is_err());
    assert!(append_entry(
        root.path(),
        DATE,
        &format!("[pdf]({})", file.path),
        Some(&other)
    )
    .is_err());
    fs::write(outside.path().join("secret.pdf"), b"external").unwrap();
    let local = root.path().join(&file.path);
    fs::remove_file(&local).unwrap();
    std::os::unix::fs::symlink(outside.path().join("secret.pdf"), &local).unwrap();
    assert!(resolve_attachment(root.path(), &id, &file.path).is_err());
}

#[test]
fn generic_image_import_keeps_bytes_and_preview_compatibility() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    let bytes = test_png();
    let file = import_file(root.path(), &id, "screen_shot.png", &bytes).unwrap();
    assert!(file.is_image);
    assert_eq!(fs::read(root.path().join(&file.path)).unwrap(), bytes);
    assert!(read_attachment(root.path(), &id, &file.path)
        .unwrap()
        .starts_with("data:image/png;base64,"));
}

#[test]
fn long_file_names_keep_their_extension() {
    let root = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    let file = import_file(
        root.path(),
        &id,
        &format!("{}.pdf", "x".repeat(240)),
        b"%PDF",
    )
    .unwrap();
    assert!(file.path.ends_with(".pdf"));
}

#[cfg(unix)]
#[test]
fn path_import_copies_regular_files_and_rejects_directories_symlinks_and_oversize() {
    let root = tempdir().unwrap();
    let source = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let id = reserve_entry_id(root.path(), DATE).unwrap();
    let pdf = source.path().join("report.pdf");
    fs::write(&pdf, b"%PDF").unwrap();
    let file = import_file_path(root.path(), &id, &pdf).unwrap();
    fs::write(&pdf, b"changed").unwrap();
    assert_eq!(fs::read(root.path().join(file.path)).unwrap(), b"%PDF");
    let link = source.path().join("symlink.pdf");
    std::os::unix::fs::symlink(&pdf, &link).unwrap();
    assert!(import_file_path(root.path(), &id, &link).is_err());
    assert!(import_file_path(root.path(), &id, source.path()).is_err());
    let large = source.path().join("large.bin");
    fs::File::create(&large)
        .unwrap()
        .set_len(20 * 1024 * 1024 + 1)
        .unwrap();
    assert!(import_file_path(root.path(), &id, &large).is_err());
}

#[cfg(unix)]
#[test]
fn delete_and_discard_refuse_symlink_asset_parent_without_touching_external_data() {
    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let saved = append_entry(root.path(), DATE, "keep", None).unwrap();
    let draft = reserve_entry_id(root.path(), DATE).unwrap();
    for id in [&saved.id, &draft] {
        fs::create_dir(outside.path().join(id)).unwrap();
        fs::write(outside.path().join(id).join("precious.txt"), "keep").unwrap();
    }
    std::os::unix::fs::symlink(outside.path(), root.path().join("assets")).unwrap();
    assert!(delete_entry(root.path(), DATE, &saved.id, "keep").is_err());
    assert!(discard_draft(root.path(), &draft).is_err());
    assert!(root.path().join(format!("{}.md", saved.id)).exists());
    for id in [&saved.id, &draft] {
        assert!(outside.path().join(id).join("precious.txt").exists());
    }
}

#[cfg(unix)]
#[test]
fn trash_recovery_refuses_symlink_asset_parent_without_touching_external_data() {
    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    initialize(root.path()).unwrap();
    let saved = append_entry(root.path(), DATE, "keep", None).unwrap();
    delete_entry(root.path(), DATE, &saved.id, "keep").unwrap();
    fs::create_dir(outside.path().join(&saved.id)).unwrap();
    fs::write(outside.path().join(&saved.id).join("precious.txt"), "keep").unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("assets")).unwrap();
    assert!(initialize(root.path()).is_err());
    assert!(outside.path().join(&saved.id).join("precious.txt").exists());
}
