# Simplify Interface Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the editable journal UI with a blank-on-focus capture surface that commits on app blur and exposes today’s items in a muted, collapsible drawer with copy actions.

**Architecture:** Add a Markdown item delimiter (`---`) and parsing helpers in Rust storage, expose new Tauri commands for listing and appending today’s items, and rebuild the frontend UI to manage capture, drawer state, and copy interactions.

**Tech Stack:** Tauri (Rust), vanilla JS/HTML/CSS.

### Task 1: Update storage tests for Markdown item parsing

**Files:**
- Modify: `src-tauri/src/storage_tests.rs`

**Step 1: Write the failing tests**
```rust
use crate::storage::{append_item_to_text, split_items};

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
```

**Step 2: Run test to verify it fails**
Run: `cargo test` in `src-tauri`
Expected: FAIL with missing `split_items`/`append_item_to_text`

**Step 3: Write minimal implementation**
(Implemented in Task 2.)

**Step 4: Run test to verify it passes**
Run: `cargo test` in `src-tauri`
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/storage.rs src-tauri/src/storage_tests.rs

git commit -m "test: add markdown item parsing coverage"
```

### Task 2: Implement Markdown item parsing + append in storage

**Files:**
- Modify: `src-tauri/src/storage.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Write the failing test**
(Already added in Task 1.)

**Step 2: Run test to verify it fails**
Run: `cargo test` in `src-tauri`
Expected: FAIL (missing helpers)

**Step 3: Write minimal implementation**
```rust
const ITEM_DIVIDER: &str = "---";

pub fn split_items(text: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in text.lines() {
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
```

**Step 4: Update existing helpers**
- Replace `count_items` logic with `split_items(text).len()`.
- Add `append_item(root, filename, item)` that loads existing file, calls `append_item_to_text`, and saves via `save_active_file`.
- Update any counts or text loading to use the new item format.

**Step 5: Add new Tauri commands**
```rust
#[derive(Serialize)]
struct TodayItems {
    filename: String,
    items: Vec<String>,
    counts: storage::Counts,
}

#[tauri::command]
fn get_today_items(app: tauri::AppHandle) -> TodayItems { /* ... */ }

#[tauri::command]
fn append_today_item(app: tauri::AppHandle, filename: String, text: String) -> storage::Counts { /* ... */ }
```
- Use `get_active_file_for_date` to compute filename and counts.
- Call `storage::split_items(&text)` for items.
- On append, call `storage::append_item(&root, &filename, &text)` and return updated counts.
- Update `invoke_handler` to include the new commands (and remove unused ones if no longer needed).

**Step 6: Run test to verify it passes**
Run: `cargo test` in `src-tauri`
Expected: PASS

**Step 7: Commit**
```bash
git add src-tauri/src/lib.rs src-tauri/src/storage.rs src-tauri/src/storage_tests.rs

git commit -m "feat: add markdown item parsing and append command"
```

### Task 3: Update the UI structure and styling

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles.css`

**Step 1: Update HTML skeleton**
```html
<div class="app">
  <section class="capture">
    <textarea id="editor" class="capture__editor" placeholder="Type..." spellcheck="true"></textarea>
  </section>

  <section class="drawer" data-state="collapsed">
    <button class="drawer__toggle" type="button">
      <span>Today</span>
      <span class="drawer__count" id="today-count">0</span>
      <span class="drawer__copy" id="copy-all" aria-hidden="true">Copy all</span>
    </button>
    <div class="drawer__panel" id="today-panel">
      <ul class="drawer__list" id="today-list"></ul>
    </div>
  </section>
</div>
```

**Step 2: Add drawer styling**
- Muted colors for drawer text and separators.
- Single-line truncation in list items with `text-overflow: ellipsis`.
- Checkbox icon via `::before` on list items.
- Collapsed/expanded transitions via `max-height` and `opacity`.

**Step 3: Commit**
```bash
git add src/index.html src/styles.css

git commit -m "feat: add capture + today drawer layout"
```

### Task 4: Update frontend behavior for capture, drawer, and copy

**Files:**
- Modify: `src/main.js`

**Step 1: Add state + rendering helpers**
```js
const state = {
  filename: "",
  items: [],
  drawerOpen: false,
  error: null,
};

function renderList() { /* build <li> per item */ }
function updateCount() { /* update badge */ }
```

**Step 2: Replace load + save flow**
- On `DOMContentLoaded`, call `get_today_items` and set `state.items`, `state.filename`, update UI.
- On `window.blur`, trim textarea, call `append_today_item` if non-empty, update state, clear textarea.
- On `window.focus`, clear textarea and focus it.

**Step 3: Implement drawer toggling**
- Toggle `data-state` on `.drawer` via `drawerOpen` and button click.

**Step 4: Implement copy actions**
- Per-item copy via `navigator.clipboard.writeText(item)`.
- Copy-all via joining items with `\n\n---\n\n`.

**Step 5: Manual verification**
- Run `npm run dev` and verify blur commit, focus clear, drawer toggle, per-item copy, copy-all.

**Step 6: Commit**
```bash
git add src/main.js

git commit -m "feat: implement blur commit + today drawer behavior"
```

### Task 5: Final verification

**Step 1: Run Rust tests**
Run: `cargo test` in `src-tauri`
Expected: PASS

**Step 2: Run frontend smoke check**
Run: `npm run dev`
Expected: App launches, blur commit works, drawer toggles, copy actions work

**Step 3: Commit (if any outstanding changes)**
```bash
git status --short
```

