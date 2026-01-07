# Simple Task Checkbox + Hover Delete Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a checkbox-driven done state with Markdown `- [x]` updates and a hover-only delete action that moves items to a rolling `trash.md`.

**Architecture:** Frontend renders checkbox state by parsing task lines and issues new Tauri commands to update or delete items. Backend updates daily files by index and appends deletions to `trash.md` with a timestamp, keeping counts consistent.

**Tech Stack:** Tauri (Rust backend), Vanilla JS/HTML/CSS frontend, Node.js `node:test` for JS unit tests, Rust unit tests in `src-tauri`.

### Task 1: Add JS task line parsing helpers with tests

**Files:**
- Create: `src/task-items.js`
- Create: `tests/task-items.test.js`
- Modify: `package.json`

**Step 1: Write the failing test**

```js
import assert from "node:assert/strict";
import test from "node:test";
import { parseTaskItem, formatTaskItem } from "../src/task-items.js";

test("parseTaskItem detects checked items", () => {
  const result = parseTaskItem("- [x] buy milk");
  assert.equal(result.checked, true);
  assert.equal(result.text, "buy milk");
});

test("parseTaskItem treats plain lines as unchecked", () => {
  const result = parseTaskItem("buy milk");
  assert.equal(result.checked, false);
  assert.equal(result.text, "buy milk");
});

test("formatTaskItem preserves multi-line content", () => {
  const result = formatTaskItem("line one\nline two", true);
  assert.equal(result, "- [x] line one\nline two");
});
```

**Step 2: Run test to verify it fails**

Run: `node --test tests/task-items.test.js`
Expected: FAIL with "Cannot find module" or missing exports.

**Step 3: Write minimal implementation**

```js
const TASK_MARKER = /^-\s\[( |x|X)\]\s+/;

export function parseTaskItem(item) {
  const lines = item.split("\n");
  const match = lines[0]?.match(TASK_MARKER);
  if (match) {
    lines[0] = lines[0].slice(match[0].length);
    return { checked: match[1].toLowerCase() === "x", text: lines.join("\n") };
  }
  return { checked: false, text: item };
}

export function formatTaskItem(text, checked) {
  const lines = text.split("\n");
  const marker = checked ? "- [x] " : "- [ ] ";
  lines[0] = marker + (lines[0] ?? "");
  return lines.join("\n");
}
```

**Step 4: Run test to verify it passes**

Run: `node --test tests/task-items.test.js`
Expected: PASS.

**Step 5: Update npm test script**

Modify `package.json`:
```json
"test": "node --test"
```

**Step 6: Run full test command**

Run: `npm test`
Expected: PASS with new tests.

**Step 7: Commit**

```bash
git add src/task-items.js tests/task-items.test.js package.json
git commit -m "test: add task item parsing helpers"
```

### Task 2: Add storage helpers + tests for update/delete/trash

**Files:**
- Modify: `src-tauri/src/storage.rs`
- Modify: `src-tauri/src/storage_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn update_item_replaces_line() {
    let root = tempdir().unwrap();
    save_active_file(root.path(), "2026-01-07.md", "first\n\n---\n\nsecond\n");
    let updated = update_item_for_date(root.path(), "2026-01-07", 0, "- [x] first");
    assert_eq!(updated.items[0], "- [x] first");
}

#[test]
fn delete_item_removes_and_appends_trash() {
    let root = tempdir().unwrap();
    save_active_file(root.path(), "2026-01-07.md", "first\n\n---\n\nsecond\n");
    let updated = delete_item_for_date(root.path(), "2026-01-07", 0);
    assert_eq!(updated.items, vec!["second"]);
    let trash = std::fs::read_to_string(root.path().join("trash.md")).unwrap();
    assert!(trash.contains("first"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p inboxapp --lib storage_tests::update_item_replaces_line`
Expected: FAIL with "cannot find function".

**Step 3: Write minimal implementation**

```rust
pub struct DayItems {
    pub date: String,
    pub items: Vec<String>,
    pub count: usize,
}

pub fn update_item_for_date(root: &Path, date: &str, index: usize, item: &str) -> DayItems {
    let filename = journal_filename(date);
    let text = load_or_create(root, &filename);
    let mut items = split_items(&text);
    if index < items.len() {
        items[index] = item.to_string();
    }
    let updated = items.join("\n\n---\n\n");
    let updated = if updated.is_empty() { updated } else { format!("{}\n", updated) };
    save_active_file(root, &filename, &updated);
    DayItems { date: date.to_string(), count: items.len(), items }
}

pub fn delete_item_for_date(root: &Path, date: &str, index: usize) -> DayItems {
    let filename = journal_filename(date);
    let text = load_or_create(root, &filename);
    let mut items = split_items(&text);
    if index < items.len() {
        let removed = items.remove(index);
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        let trash_entry = format!("[{}]\n{}", timestamp, removed);
        let trash_path = root.join("trash.md");
        let trash_text = std::fs::read_to_string(&trash_path).unwrap_or_default();
        let trash_updated = append_item_to_text(&trash_text, &trash_entry);
        let _ = std::fs::write(trash_path, trash_updated);
    }
    let updated = items.join("\n\n---\n\n");
    let updated = if updated.is_empty() { updated } else { format!("{}\n", updated) };
    save_active_file(root, &filename, &updated);
    DayItems { date: date.to_string(), count: items.len(), items }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p inboxapp --lib storage_tests::update_item_replaces_line`
Expected: PASS.

**Step 5: Run full Rust tests**

Run: `cargo test -p inboxapp --lib`
Expected: PASS.

**Step 6: Commit**

```bash
git add src-tauri/src/storage.rs src-tauri/src/storage_tests.rs
git commit -m "feat: update and delete items in storage"
```

### Task 3: Expose update/delete commands to the frontend

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Step 1: Write minimal command signatures**

```rust
#[tauri::command]
fn update_item_for_date(app: tauri::AppHandle, date: String, index: usize, item: String) -> storage::DayItems {
    let root = storage::storage_root();
    let updated = storage::update_item_for_date(&root, &date, index, &item);
    if date == today_string() {
        set_window_title(&app, updated.count);
    }
    updated
}

#[tauri::command]
fn delete_item_for_date(app: tauri::AppHandle, date: String, index: usize) -> storage::DayItems {
    let root = storage::storage_root();
    let updated = storage::delete_item_for_date(&root, &date, index);
    if date == today_string() {
        set_window_title(&app, updated.count);
    }
    updated
}
```

**Step 2: Wire into invoke handler**

Add to `tauri::generate_handler![ ... ]` list.

**Step 3: Run Rust tests**

Run: `cargo test -p inboxapp --lib`
Expected: PASS.

**Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: expose update/delete commands"
```

### Task 4: Update drawer UI to show checkboxes and delete action

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles.css`

**Step 1: Add checkbox + delete button styles**

Add a checkbox container and delete button styles (hidden until hover) alongside existing copy styles.

**Step 2: Update markup if needed**

Ensure list items can render a checkbox input and delete button.

**Step 3: Verify layout manually**

Run: `npm run dev`
Expected: Drawer list shows checkbox and hover actions without layout shift.

**Step 4: Commit**

```bash
git add src/index.html src/styles.css
git commit -m "feat: add checkbox + delete styles"
```

### Task 5: Wire checkbox + delete behavior in JS

**Files:**
- Modify: `src/main.js`
- Modify: `src/task-items.js`

**Step 1: Render checkbox state and delete button**

Update `renderList()` to:
- Parse each item for checked state and display text.
- Render a checkbox input (checked based on parse result).
- Add delete button next to copy.

**Step 2: Implement checkbox handler**

On toggle:
- Build updated item with `formatTaskItem()`.
- Call `invoke("update_item_for_date", { date: state.viewDate, index, item: updatedItem })`.
- Update `state.viewItems[index]` on success and re-render.

**Step 3: Implement delete handler**

On delete:
- Call `invoke("delete_item_for_date", { date: state.viewDate, index })`.
- Update `state.viewItems` with returned items, update counts/day strip, and re-render.

**Step 4: Run JS tests**

Run: `node --test tests/task-items.test.js`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/main.js src/task-items.js
git commit -m "feat: support checkbox completion and delete"
```

### Task 6: End-to-end verification

**Step 1: Run tests**

Run: `npm test`
Expected: PASS.

Run: `cargo test -p inboxapp --lib`
Expected: PASS.

**Step 2: Manual smoke test**

Run: `npm run dev`
Expected:
- Checkbox toggles update the file to `- [x]` / `- [ ]`.
- Deleted items vanish and appear in `~/.inboxapp/trash.md`.
- Copy includes task markers.

**Step 3: Commit any remaining changes**

```bash
git add docs/plans/2026-01-07-simple-task-checkbox-delete-design.md docs/plans/2026-01-07-simple-task-checkbox-delete-implementation-plan.md
git commit -m "docs: add simple checkbox/delete plan"
```
