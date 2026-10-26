use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: u32 = 2;
const BACKUP: &str = "legacy-backup-v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Attachment {
    pub path: String,
    pub name: String,
    pub is_image: bool,
}
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
    pub items: Vec<Entry>,
    pub count: usize,
}
#[derive(Serialize, Deserialize)]
struct Config {
    schema_version: u32,
    #[serde(default)]
    next_sequence: BTreeMap<String, u64>,
}
#[derive(Serialize, Deserialize)]
struct Migration {
    sources: Vec<LegacySource>,
}
#[derive(Serialize, Deserialize)]
struct LegacySource {
    filename: String,
    original: String,
    entries: Vec<Entry>,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
pub fn storage_root() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".inboxapp")
}
pub fn validate_date(date: &str) -> io::Result<()> {
    if date.len() != 10
        || NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map(|d| d.format("%Y-%m-%d").to_string() != date)
            .unwrap_or(true)
    {
        return Err(invalid("Invalid entry date"));
    }
    Ok(())
}
pub fn id_date(id: &str) -> io::Result<&str> {
    if !id.is_ascii()
        || id.len() < 15
        || id.as_bytes()[10] != b'-'
        || !id[11..].bytes().all(|b| b.is_ascii_digit())
        || id[11..].parse::<u64>().is_err()
    {
        return Err(invalid("Invalid entry ID"));
    }
    validate_date(&id[..10])?;
    Ok(&id[..10])
}
fn check_id_date(id: &str, date: &str) -> io::Result<()> {
    validate_date(date)?;
    if id_date(id)? != date {
        return Err(invalid("Entry ID does not belong to this day"));
    }
    Ok(())
}
fn atomic_write(path: &Path, bytes: &[u8], replace: bool) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("Missing parent directory"))?;
    fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    if replace {
        file.persist(path).map_err(|e| e.error)?;
    } else {
        file.persist_noclobber(path).map_err(|e| e.error)?;
    }
    // Persist directory entries as well as contents before reporting success.
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}
fn body(text: &str) -> String {
    format!("{}\n", text.trim_end())
}
fn read_entry(root: &Path, id: &str) -> io::Result<Entry> {
    id_date(id)?;
    Ok(Entry {
        id: id.into(),
        text: fs::read_to_string(root.join(format!("{id}.md")))?
            .trim_end()
            .into(),
    })
}

/// Only used by the one-time legacy migration. New entries may contain any Markdown.
pub fn split_items(text: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.lines() {
        if current.is_empty() && line.trim().is_empty() {
            continue;
        }
        if line.trim() == "---" {
            let item = current.join("\n").trim_end().to_string();
            if !item.trim().is_empty() {
                items.push(item);
            }
            current.clear();
        } else {
            current.push(line);
        }
    }
    let item = current.join("\n").trim_end().to_string();
    if !item.trim().is_empty() {
        items.push(item);
    }
    items
}

pub fn initialize(root: &Path) -> io::Result<()> {
    fs::create_dir_all(root)?;
    let config_path = root.join("config.yml");
    match fs::read_to_string(&config_path) {
        Ok(text) => {
            let config: Config = serde_yaml::from_str(&text)
                .map_err(|e| invalid(format!("Cannot read config.yml: {e}")))?;
            if config.schema_version != SCHEMA_VERSION {
                return Err(invalid(format!("Unsupported storage schema {}; this app supports {}. Your files were not changed.", config.schema_version, SCHEMA_VERSION)));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => migrate_legacy(root)?,
        Err(error) => return Err(error),
    }
    recover_trash(root)
}

fn migrate_legacy(root: &Path) -> io::Result<()> {
    let backup = root.join(BACKUP);
    let manifest_path = backup.join("migration.json");
    let manifest: Migration = if manifest_path.exists() {
        serde_json::from_slice(&fs::read(&manifest_path)?)
            .map_err(|e| invalid(format!("Invalid migration manifest: {e}")))?
    } else {
        let mut sources = Vec::new();
        let mut paths: Vec<_> = fs::read_dir(root)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|e| e.path())
            .collect();
        paths.sort();
        for path in paths {
            let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some(date) = filename.strip_suffix(".md") else {
                continue;
            };
            if filename != "trash.md" && validate_date(date).is_err() {
                continue;
            }
            let original = fs::read_to_string(&path)?;
            let entries = if filename == "trash.md" {
                Vec::new()
            } else {
                let mut next = next_sequence(root, date)?;
                split_items(&original)
                    .into_iter()
                    .map(|text| {
                        let entry = Entry {
                            id: format!("{date}-{next:04}"),
                            text,
                        };
                        next += 1;
                        entry
                    })
                    .collect()
            };
            sources.push(LegacySource {
                filename: filename.into(),
                original,
                entries,
            });
        }
        let manifest = Migration { sources };
        fs::create_dir_all(&backup)?;
        // Immutable originals and the mapping are persisted before any entry is created.
        for source in &manifest.sources {
            let target = backup.join(&source.filename);
            if target.exists() {
                if fs::read_to_string(&target)? != source.original {
                    return Err(invalid(
                        "Legacy backup already exists with different contents",
                    ));
                }
            } else {
                atomic_write(&target, source.original.as_bytes(), false)?;
            }
        }
        atomic_write(
            &manifest_path,
            &serde_json::to_vec_pretty(&manifest).map_err(invalid_json)?,
            false,
        )?;
        manifest
    };
    // Validate all originals first. Never merge a changed daily file into a partial upgrade.
    for source in &manifest.sources {
        let backup_text = fs::read_to_string(backup.join(&source.filename))?;
        if backup_text != source.original {
            return Err(invalid("Legacy backup changed during upgrade"));
        }
        match fs::read_to_string(root.join(&source.filename)) {
            Ok(text) if text != source.original => return Err(invalid(
                "A daily file changed during upgrade; originals are retained in legacy-backup-v1",
            )),
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    for source in &manifest.sources {
        for entry in &source.entries {
            let path = root.join(format!("{}.md", entry.id));
            let text = body(&entry.text);
            if path.exists() {
                if fs::read_to_string(&path)? != text {
                    return Err(invalid("An entry conflicts with the pending upgrade; no existing file was overwritten"));
                }
            } else {
                atomic_write(&path, text.as_bytes(), false)?;
            }
        }
    }
    // All copies verified before removing the old active journals; backup remains permanently.
    for source in &manifest.sources {
        let path = root.join(&source.filename);
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    let mut config = Config {
        schema_version: SCHEMA_VERSION,
        next_sequence: BTreeMap::new(),
    };
    for id in entry_ids(root)? {
        let date = id_date(&id)?;
        config
            .next_sequence
            .insert(date.into(), next_sequence(root, date)?);
    }
    let yaml = serde_yaml::to_string(&config).map_err(|e| invalid(e.to_string()))?;
    atomic_write(&root.join("config.yml"), yaml.as_bytes(), false)
}
fn invalid_json(error: serde_json::Error) -> io::Error {
    invalid(error.to_string())
}

fn entry_ids(root: &Path) -> io::Result<Vec<String>> {
    let mut ids: Vec<String> = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let filename = entry.file_name();
        let Some(id) = filename.to_str().and_then(|s| s.strip_suffix(".md")) else {
            continue;
        };
        if id_date(id).is_ok() && entry.file_type()?.is_file() {
            ids.push(id.into());
        }
    }
    ids.sort_by_key(|id| (id[..10].to_string(), id[11..].parse::<u64>().unwrap_or(0)));
    Ok(ids)
}
fn next_sequence(root: &Path, date: &str) -> io::Result<u64> {
    validate_date(date)?;
    let mut max = 0;
    for dir in [
        root.to_path_buf(),
        root.join(".drafts"),
        root.join(".trash"),
        root.join("assets"),
    ] {
        match fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry?;
                    // Directories named *.md aren't entries; later writes should surface their error.
                    if dir == root && !entry.file_type()?.is_file() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    let id = name
                        .strip_suffix(".md")
                        .unwrap_or(&name)
                        .strip_prefix(".pending-")
                        .unwrap_or(name.strip_suffix(".md").unwrap_or(&name));
                    if id_date(id).ok() == Some(date) {
                        max = max.max(
                            id[11..]
                                .parse::<u64>()
                                .map_err(|_| invalid("Entry sequence exhausted"))?,
                        );
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    max.checked_add(1)
        .ok_or_else(|| invalid("Entry sequence exhausted"))
}

pub fn reserve_entry_id(root: &Path, date: &str) -> io::Result<String> {
    let mut config: Config = serde_yaml::from_str(&fs::read_to_string(root.join("config.yml"))?)
        .map_err(|e| invalid(e.to_string()))?;
    let mut sequence =
        next_sequence(root, date)?.max(*config.next_sequence.get(date).unwrap_or(&1));
    fs::create_dir_all(root.join(".drafts"))?;
    loop {
        let id = format!("{date}-{sequence:04}");
        match atomic_write(&root.join(".drafts").join(&id), b"reserved\n", false) {
            Ok(()) => {
                config.next_sequence.insert(
                    date.into(),
                    sequence
                        .checked_add(1)
                        .ok_or_else(|| invalid("Entry sequence exhausted"))?,
                );
                let yaml = serde_yaml::to_string(&config).map_err(|e| invalid(e.to_string()))?;
                atomic_write(&root.join("config.yml"), yaml.as_bytes(), true)?;
                return Ok(id);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                sequence = sequence
                    .checked_add(1)
                    .ok_or_else(|| invalid("Entry sequence exhausted"))?
            }
            Err(e) => return Err(e),
        }
    }
}

pub fn read_items_for_date(root: &Path, date: &str) -> io::Result<Vec<Entry>> {
    validate_date(date)?;
    entry_ids(root)?
        .into_iter()
        .filter(|id| &id[..10] == date)
        .map(|id| read_entry(root, &id))
        .collect()
}
pub fn day_items(root: &Path, date: &str) -> io::Result<DayItems> {
    let items = read_items_for_date(root, date)?;
    Ok(DayItems {
        date: date.into(),
        count: items.len(),
        items,
    })
}
pub fn list_day_counts(root: &Path) -> io::Result<Vec<DayCount>> {
    let mut days = BTreeMap::new();
    for id in entry_ids(root)? {
        *days.entry(id[..10].to_string()).or_insert(0) += 1;
    }
    Ok(days
        .into_iter()
        .map(|(date, count)| DayCount { date, count })
        .collect())
}
pub fn counts(root: &Path, date: &str) -> io::Result<Counts> {
    let ids = entry_ids(root)?;
    Ok(Counts {
        current: ids.iter().filter(|id| &id[..10] == date).count(),
        total: ids.len(),
        files: ids.len(),
    })
}

pub fn append_entry(
    root: &Path,
    date: &str,
    text: &str,
    requested_id: Option<&str>,
) -> io::Result<Entry> {
    validate_date(date)?;
    if text.trim().is_empty() {
        return Err(invalid("Cannot save an empty entry"));
    }
    let id = match requested_id {
        Some(id) => {
            check_id_date(id, date)?;
            id.to_string()
        }
        None => reserve_entry_id(root, date)?,
    };
    validate_asset_ownership(&id, text)?;
    let entry = Entry {
        id: id.clone(),
        text: text.trim_end().into(),
    };
    let target = root.join(format!("{id}.md"));
    if target.exists() {
        if read_entry(root, &id)? == entry {
            return Ok(entry);
        }
        return Err(invalid("Entry already exists with different contents"));
    }
    if !root.join(".drafts").join(&id).is_file() {
        return Err(invalid("Draft reservation no longer exists"));
    }
    atomic_write(&target, body(text).as_bytes(), false)?;
    // Reservation removal is best effort: stale reservations are harmless and preserve uniqueness.
    let _ = fs::remove_file(root.join(".drafts").join(&id));
    Ok(entry)
}
pub fn update_entry(
    root: &Path,
    date: &str,
    id: &str,
    text: &str,
    expected: &str,
) -> io::Result<DayItems> {
    check_id_date(id, date)?;
    if text.trim().is_empty() {
        return Err(invalid("Cannot save an empty entry"));
    }
    validate_asset_ownership(id, text)?;
    let mut result = day_items(root, date)?;
    if read_entry(root, id)?.text != expected {
        return Err(invalid("Entry changed elsewhere; reopen it before saving"));
    }
    atomic_write(&root.join(format!("{id}.md")), body(text).as_bytes(), true)?;
    for entry in &mut result.items {
        if entry.id == id {
            entry.text = text.trim_end().into();
        }
    }
    Ok(result)
}

fn copy_tree(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    if let Some(parent) = target.parent() {
        fs::File::open(parent)?.sync_all()?;
    }
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let destination = target.join(entry.file_name());
        if kind.is_symlink() {
            return Err(invalid(
                "Attachment folders must not contain symbolic links",
            ));
        }
        if kind.is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else if kind.is_file() {
            atomic_write(&destination, &fs::read(entry.path())?, true)?;
        } else {
            return Err(invalid("Unsupported file in attachment folder"));
        }
    }
    fs::File::open(target)?.sync_all()?;
    Ok(())
}
fn finish_trash(root: &Path, id: &str) -> io::Result<()> {
    id_date(id)?;
    let trash = root.join(".trash");
    let pending = trash.join(format!(".pending-{id}"));
    let final_path = trash.join(id);
    let name = format!("{id}.md");
    if pending.join(&name).is_file() {
        if final_path.exists() {
            return Err(invalid("Trash entry already exists"));
        }
        fs::rename(&pending, &final_path)?;
        fs::File::open(&trash)?.sync_all()?;
    }
    // The committed bundle is the recovery marker; only clean assets if active entry is gone.
    if final_path.join(&name).is_file() && !root.join(&name).exists() {
        if let Some(assets) = asset_directory(root, id, false)? {
            fs::remove_dir_all(assets)?;
        }
        let reservation = root.join(".drafts").join(id);
        if reservation.exists() {
            fs::remove_file(reservation)?;
        }
    }
    Ok(())
}
fn recover_trash(root: &Path) -> io::Result<()> {
    let trash = root.join(".trash");
    if !trash.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(trash)? {
        let name = entry?.file_name().to_string_lossy().to_string();
        let id = name.strip_prefix(".pending-").unwrap_or(&name);
        if id_date(id).is_ok() {
            finish_trash(root, id)?;
        }
    }
    Ok(())
}
pub fn delete_entry(root: &Path, date: &str, id: &str, expected: &str) -> io::Result<DayItems> {
    check_id_date(id, date)?;
    let mut result = day_items(root, date)?;
    if read_entry(root, id)?.text != expected {
        return Err(invalid(
            "Entry changed elsewhere; reopen it before deleting",
        ));
    }
    let pending = root.join(".trash").join(format!(".pending-{id}"));
    if root.join(".trash").join(id).exists() {
        return Err(invalid("Trash entry already exists"));
    }
    fs::create_dir_all(&pending)?;
    if let Some(assets) = asset_directory(root, id, false)? {
        copy_tree(&assets, &pending.join("assets").join(id))?;
    }
    if read_entry(root, id)?.text != expected {
        return Err(invalid(
            "Entry changed elsewhere; reopen it before deleting",
        ));
    }
    if pending.join("assets").exists() {
        fs::File::open(pending.join("assets"))?.sync_all()?;
    }
    fs::File::open(&pending)?.sync_all()?;
    fs::File::open(root.join(".trash"))?.sync_all()?;
    fs::File::open(root)?.sync_all()?;
    // Until this rename succeeds, the active Markdown and its assets remain untouched.
    fs::rename(
        root.join(format!("{id}.md")),
        pending.join(format!("{id}.md")),
    )?;
    fs::File::open(root)?.sync_all()?;
    fs::File::open(&pending)?.sync_all()?;
    finish_trash(root, id)?;
    result.items.retain(|entry| entry.id != id);
    result.count = result.items.len();
    Ok(result)
}
pub fn discard_draft(root: &Path, id: &str) -> io::Result<()> {
    id_date(id)?;
    if root.join(format!("{id}.md")).exists() {
        return Err(invalid("Cannot discard a saved entry"));
    }
    let reservation = root.join(".drafts").join(id);
    if !reservation.is_file() {
        return Err(invalid("Draft reservation no longer exists"));
    }
    if let Some(assets) = asset_directory(root, id, false)? {
        fs::remove_dir_all(assets)?;
    }
    fs::remove_file(reservation)
}

const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
const MAX_IMAGE_PIXELS: u64 = 40_000_000;

fn validate_image(bytes: &[u8]) -> io::Result<(&'static str, &'static str)> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(invalid("Image exceeds the 20 MB limit"));
    }
    let format =
        image::guess_format(bytes).map_err(|_| invalid("Choose a PNG, JPEG or WebP image"))?;
    let kind = match format {
        image::ImageFormat::Png => ("png", "image/png"),
        image::ImageFormat::Jpeg => ("jpg", "image/jpeg"),
        image::ImageFormat::WebP => ("webp", "image/webp"),
        _ => return Err(invalid("Choose a PNG, JPEG or WebP image")),
    };
    let dimensions = image::ImageReader::with_format(io::Cursor::new(bytes), format)
        .into_dimensions()
        .map_err(|e| invalid(format!("Cannot read image: {e}")))?;
    if dimensions.0 == 0
        || dimensions.1 == 0
        || u64::from(dimensions.0) * u64::from(dimensions.1) > MAX_IMAGE_PIXELS
    {
        return Err(invalid("Image exceeds the 40 megapixel limit"));
    }
    let mut reader = image::ImageReader::with_format(io::Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16000);
    limits.max_image_height = Some(16000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    reader
        .decode()
        .map_err(|e| invalid(format!("Cannot decode image: {e}")))?;
    Ok(kind)
}

// Generated links remain owned by one entry. A raw Markdown copy cannot silently
// borrow another entry's attachment because processing that original would break it.
fn validate_asset_ownership(id: &str, text: &str) -> io::Result<()> {
    for remainder in text.split("assets/").skip(1) {
        if let Some((owner, _)) = remainder.split_once('/') {
            if id_date(owner).is_ok() && owner != id {
                return Err(invalid("This attachment belongs to another entry. Attach a copy to this entry instead."));
            }
        }
    }
    Ok(())
}

// This check also guards destructive cleanup: checking only the final directory
// would follow a symlink in the assets parent into an unrelated external folder.
fn asset_directory(root: &Path, id: &str, create: bool) -> io::Result<Option<PathBuf>> {
    id_date(id)?;
    let mut path = root.to_path_buf();
    for component in ["assets", id] {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(invalid(
                    "Attachment directory must be an ordinary local folder",
                ))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound && create => {
                fs::create_dir(&path)?;
                fs::File::open(path.parent().unwrap())?.sync_all()?;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        }
    }
    Ok(Some(path))
}

fn owned_asset_dir(root: &Path, id: &str, create: bool) -> io::Result<PathBuf> {
    id_date(id)?;
    if !root.join(format!("{id}.md")).is_file() && !root.join(".drafts").join(id).is_file() {
        return Err(invalid("Entry or draft no longer exists"));
    }
    asset_directory(root, id, create)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Attachment folder no longer exists",
        )
    })
}

pub fn import_attachment(root: &Path, id: &str, bytes: &[u8]) -> io::Result<String> {
    let (extension, _) = validate_image(bytes)?;
    let directory = owned_asset_dir(root, id, true)?;
    let mut sequence = 1u64;
    loop {
        let name = format!("image-{sequence:04}.{extension}");
        match atomic_write(&directory.join(&name), bytes, false) {
            Ok(()) => return Ok(format!("assets/{id}/{name}")),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                sequence = sequence
                    .checked_add(1)
                    .ok_or_else(|| invalid("Too many attachments"))?
            }
            Err(e) => return Err(e),
        }
    }
}

pub fn import_file(root: &Path, id: &str, name: &str, bytes: &[u8]) -> io::Result<Attachment> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(invalid("File exceeds the 20 MB limit"));
    }
    let directory = owned_asset_dir(root, id, true)?;
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
        .collect();
    let safe = safe.trim_matches('.');
    let safe = if safe.is_empty() { "attachment" } else { safe };
    let safe = if safe.len() > 180 {
        match safe.rsplit_once('.') {
            Some((stem, extension)) if extension.len() < 80 => format!(
                "{}.{}",
                &stem[..(179 - extension.len()).min(stem.len())],
                extension
            ),
            _ => safe[..180].to_string(),
        }
    } else {
        safe.to_string()
    };
    let safe = safe.as_str();
    let (stem, extension) = match safe.rsplit_once('.') {
        Some((stem, extension)) => (stem, format!(".{extension}")),
        None => (safe, String::new()),
    };
    for sequence in 1u64.. {
        let filename = if sequence == 1 {
            safe.to_string()
        } else {
            format!("{stem}-{sequence}{extension}")
        };
        match atomic_write(&directory.join(&filename), bytes, false) {
            Ok(()) => {
                return Ok(Attachment {
                    path: format!("assets/{id}/{filename}"),
                    name: if name.is_empty() {
                        "attachment".into()
                    } else {
                        name.into()
                    },
                    is_image: validate_image(bytes).is_ok(),
                })
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(invalid("Too many attachments"))
}

pub fn import_file_path(root: &Path, id: &str, source: &Path) -> io::Result<Attachment> {
    use std::io::Read;
    let metadata = fs::symlink_metadata(source)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(invalid(
            "Choose regular files, not folders or symbolic links",
        ));
    }
    if metadata.len() > MAX_IMAGE_BYTES as u64 {
        return Err(invalid("File exceeds the 20 MB limit"));
    }
    let mut bytes = Vec::new();
    fs::File::open(source)?
        .take(MAX_IMAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let name = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("attachment");
    import_file(root, id, name, &bytes)
}

pub fn resolve_attachment(root: &Path, id: &str, relative: &str) -> io::Result<PathBuf> {
    let directory = owned_asset_dir(root, id, false)?;
    let prefix = format!("assets/{id}/");
    let name = relative
        .strip_prefix(&prefix)
        .ok_or_else(|| invalid("Attachment belongs to another entry"))?;
    if name.is_empty()
        || name == "."
        || name == ".."
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'.' || c == b'_')
    {
        return Err(invalid("Invalid attachment path"));
    }
    let path = directory.join(name);
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_IMAGE_BYTES as u64
    {
        return Err(invalid(
            "Attachment must be a regular local file under 20 MB",
        ));
    }
    Ok(path)
}

pub fn read_attachment(root: &Path, id: &str, relative: &str) -> io::Result<String> {
    use base64::Engine;
    let bytes = fs::read(resolve_attachment(root, id, relative)?)?;
    let (_, mime) = validate_image(&bytes)?;
    Ok(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

pub fn encode_clipboard_image(width: u32, height: u32, rgba: Vec<u8>) -> io::Result<Vec<u8>> {
    let pixels = u64::from(width) * u64::from(height);
    if pixels == 0
        || pixels > MAX_IMAGE_PIXELS
        || width > 16000
        || height > 16000
        || pixels * 4 != rgba.len() as u64
    {
        return Err(invalid(
            "Clipboard image is invalid or exceeds the 40 megapixel limit",
        ));
    }
    let image = image::RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| invalid("Invalid clipboard image"))?;
    let mut buffer = io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut buffer, image::ImageFormat::Png)
        .map_err(|e| invalid(format!("Cannot encode clipboard image: {e}")))?;
    let bytes = buffer.into_inner();
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(invalid("Image exceeds the 20 MB limit"));
    }
    Ok(bytes)
}
