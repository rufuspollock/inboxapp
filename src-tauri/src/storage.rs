use std::path::{Path, PathBuf};

pub fn storage_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".inboxapp")
}

pub fn journal_filename(date: &str) -> String {
    format!("{date}.md")
}

pub fn split_archived(text: &str) -> (Vec<String>, Vec<String>) {
    let mut active = Vec::new();
    let mut archived = Vec::new();
    let mut in_archived = false;

    for line in text.lines() {
        if line.trim_end() == "## Archived" {
            in_archived = true;
            continue;
        }
        if in_archived {
            archived.push(line.to_string());
        } else {
            active.push(line.to_string());
        }
    }

    (active, archived)
}

pub fn archive_line(text: &str, line_idx: usize) -> String {
    let (mut active, mut archived) = split_archived(text);
    let mut seen = 0;
    let mut target = None;

    for (idx, line) in active.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if seen == line_idx {
            target = Some(idx);
            break;
        }
        seen += 1;
    }

    if let Some(idx) = target {
        let line = active.remove(idx);
        if !line.trim().is_empty() {
            archived.push(line);
        }
    } else {
        return text.to_string();
    }

    let mut out = String::new();
    for line in active {
        out.push_str(&line);
        out.push('\n');
    }

    if !archived.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("## Archived\n");
        for line in archived {
            out.push_str(&line);
            out.push('\n');
        }
    }

    out
}

pub fn count_items(text: &str) -> usize {
    let (active, _) = split_archived(text);
    active
        .iter()
        .filter(|line| !line.trim().is_empty())
        .count()
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
                files.push(name.to_string());
            }
        }
    }
    files.sort();
    files
}
