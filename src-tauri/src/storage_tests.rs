use crate::storage::{archive_line, journal_filename};

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
