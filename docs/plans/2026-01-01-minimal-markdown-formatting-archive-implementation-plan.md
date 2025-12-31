# Minimal Markdown Formatting + Archive Toggle Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a layered renderer that visually formats bold/italic and clickable checklist items, with backend archiving + restore and a bottom toggle to show archived items inline.

**Architecture:** Keep the `textarea` as the source of truth and render a read-only HTML layer for visual formatting. Archive/restore actions call new Rust commands that move lines between active and `## Archived`, verifying line text to avoid stale updates.

**Tech Stack:** Tauri (Rust backend), vanilla HTML/CSS/JS frontend.

## Task 1: Add storage tests for archive/restore with text verification

**Files:**
- Modify: `src-tauri/src/storage_tests.rs`

**Step 1: Write failing tests for archive/restore with text matching**

```rust
#[test]
fn archive_line_requires_matching_text() {
    let input = "- one\n- two\n";
    let output = archive_line_matching(input, 0, "- nope");
    assert!(output.is_err());
}

#[test]
fn archive_line_appends_archived_header_when_missing() {
    let input = "- one\n- two\n";
    let output = archive_line_matching(input, 1, "- two").unwrap();
    assert!(output.contains("## Archived"));
    assert!(output.contains("- two"));
}

#[test]
fn restore_line_moves_item_back_to_active() {
    let input = "- one\n\n## Archived\n- done\n";
    let output = restore_line_matching(input, 0, "- done").unwrap();
    assert!(output.contains("- one"));
    assert!(output.contains("- done"));
    assert!(!output.contains("## Archived\n- done\n"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p inboxapp --test storage_tests`
Expected: FAIL with missing functions (`archive_line_matching`, `restore_line_matching`).

**Step 3: Commit test additions**

```bash
git add src-tauri/src/storage_tests.rs
git commit -m "test: cover archive/restore with text matching"
```

## Task 2: Implement archive/restore helpers in storage

**Files:**
- Modify: `src-tauri/src/storage.rs`

**Step 1: Implement helpers**

Add a small error type and helpers that use line index (0-based within active/archived lines, including blanks) and verify the provided line text:

```rust
#[derive(Debug)]
pub enum ArchiveError {
    LineNotFound,
    TextMismatch,
}

pub fn archive_line_matching(text: &str, line_idx: usize, line_text: &str) -> Result<String, ArchiveError> {
    // split active/archived, find active line by index, verify text, move to archive
}

pub fn restore_line_matching(text: &str, line_idx: usize, line_text: &str) -> Result<String, ArchiveError> {
    // find archived line by index, verify text, move above archive header
}
```

**Step 2: Keep existing behavior working**

Update `archive_line` to call `archive_line_matching` without text verification or keep it for legacy if still needed by other call sites. Ensure `count_items` still ignores archived lines.

**Step 3: Run tests**

Run: `cargo test -p inboxapp --test storage_tests`
Expected: PASS.

**Step 4: Commit**

```bash
git add src-tauri/src/storage.rs
git commit -m "feat: add archive/restore helpers with verification"
```

## Task 3: Add Tauri commands for archive/restore with verification

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add new command for restore**

Add a `restore_item` command mirroring `archive_item`:

```rust
#[tauri::command]
fn restore_item(app: tauri::AppHandle, filename: String, line_idx: usize, line_text: String) -> Result<ArchiveResult, String> {
    let root = storage::storage_root();
    let text = storage::load_or_create(&root, &filename);
    let updated = storage::restore_line_matching(&text, line_idx, &line_text)
        .map_err(|_| "restore failed".to_string())?;
    let counts = storage::save_active_file(&root, &filename, &updated);
    set_tray_title(&app, counts.current);
    set_window_title(&app, counts.current);
    Ok(ArchiveResult { text: updated, counts })
}
```

Update `archive_item` to accept `line_text` and use `archive_line_matching` with an error return on mismatch.

**Step 2: Wire into invoke handler**

Add `restore_item` to `tauri::generate_handler![...]`.

**Step 3: Run tests**

Run: `cargo test -p inboxapp --test storage_tests`
Expected: PASS.

**Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add restore_item command"
```

## Task 4: Add renderer + toggle structure in HTML and CSS

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles.css`

**Step 1: Update HTML**

Add a renderer layer and a toggle control:

```html
<div class="app">
  <div class="app__editor-wrap">
    <div id="render" class="app__render" aria-hidden="true"></div>
    <textarea id="editor" class="app__editor" placeholder="Type..." spellcheck="true"></textarea>
  </div>
  <button id="toggle-archive" class="app__toggle" type="button">Show archived</button>
</div>
```

**Step 2: Update CSS for layering + toggle**

Add styles for `.app__editor-wrap`, `.app__render`, `.app__toggle`, and checkbox styling. Ensure textarea text is transparent while caret remains visible:

```css
.app__editor-wrap { position: relative; max-width: 720px; margin: 0 auto; }
.app__render { position: absolute; inset: 0; white-space: pre-wrap; font-size: 17px; line-height: 1.6; }
.app__editor { position: relative; z-index: 2; color: transparent; caret-color: var(--ink); background: transparent; }
.app__toggle { position: sticky; bottom: 16px; margin: 24px auto 0; display: block; }
```

**Step 3: Commit**

```bash
git add src/index.html src/styles.css
git commit -m "feat: add renderer layer and archive toggle"
```

## Task 5: Implement renderer + archive/restore interactions in JS

**Files:**
- Modify: `src/main.js`

**Step 1: Add renderer helpers**

Add helpers to split active/archived, render bold/italic, and attach checkbox/restore controls:

```js
function splitArchived(text) {
  const lines = text.split("\n");
  const idx = lines.findIndex((line) => line.trimEnd() === "## Archived");
  if (idx === -1) return { active: lines, archived: [] };
  return { active: lines.slice(0, idx), archived: lines.slice(idx + 1) };
}

function formatLine(line) {
  // replace **bold** and *italic* with spans but keep markers
}
```

**Step 2: Render active + archived**

Render active lines to `#render`, using data attributes:

```js
function renderEditor(text) {
  const { active, archived } = splitArchived(text);
  // build HTML with line indexes
}
```

**Step 3: Wire archive clicks**

On checkbox click:
- send `invoke("archive_item", { filename, lineIdx, lineText })`
- update editor text with response and re-render.

**Step 4: Toggle archived + restore**

- Add state `showArchived`.
- Toggle button sets label to `Hide archived`/`Show archived`.
- Render archived block when enabled; restore button calls `restore_item` with archived line index + text.

**Step 5: Scroll sync**

Sync `scrollTop` from textarea to renderer on scroll.

**Step 6: Commit**

```bash
git add src/main.js
git commit -m "feat: render formatted text with archive interactions"
```

## Task 6: Manual verification

**Step 1: Run the app**

Run: `npm run dev`

**Step 2: Verify behavior**
- Typing remains fast and cursor visible.
- `**bold**` and `*italic*` appear visually formatted but markers remain.
- `- [ ] item` shows a checkbox; clicking archives the line.
- Toggle shows archived items inline; restore returns them to active list.
- Window title count updates after archive/restore.

**Step 3: Commit any fixes**

```bash
git add src/main.js src/styles.css src/index.html src-tauri/src/lib.rs src-tauri/src/storage.rs

git commit -m "fix: polish rendering and archive toggle"
```
