use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Counts {
    pub current: usize,
    pub total: usize,
    pub files: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveFile {
    pub filename: String,
    pub text: String,
    pub counts: Counts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveError {
    LineNotFound,
    TextMismatch,
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

pub fn archive_line_matching(
    text: &str,
    line_idx: usize,
    line_text: &str,
) -> Result<String, ArchiveError> {
    let (mut active, mut archived) = split_archived(text);

    if line_idx >= active.len() {
        return Err(ArchiveError::LineNotFound);
    }

    let target_line = active.get(line_idx).cloned().unwrap_or_default();
    if target_line != line_text {
        return Err(ArchiveError::TextMismatch);
    }
    if target_line.trim().is_empty() {
        return Err(ArchiveError::LineNotFound);
    }

    active.remove(line_idx);
    archived.push(target_line);

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

    Ok(out)
}

pub fn restore_line_matching(
    text: &str,
    line_idx: usize,
    line_text: &str,
) -> Result<String, ArchiveError> {
    let (mut active, mut archived) = split_archived(text);

    if line_idx >= archived.len() {
        return Err(ArchiveError::LineNotFound);
    }

    let target_line = archived.get(line_idx).cloned().unwrap_or_default();
    if target_line != line_text {
        return Err(ArchiveError::TextMismatch);
    }
    if target_line.trim().is_empty() {
        return Err(ArchiveError::LineNotFound);
    }

    archived.remove(line_idx);
    active.push(target_line);

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

    Ok(out)
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
