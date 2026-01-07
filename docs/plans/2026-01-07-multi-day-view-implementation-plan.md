# Multi-day View Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a minimal multi-day view with a bottom day strip that lets users view past days while the editor continues to write to today.

**Architecture:** Keep two date contexts in the frontend: `todayDate` (write target) and `viewDate` (what the drawer shows). Add backend helpers to fetch items for a specific date without creating files, plus day counts for the strip. Window title reflects `viewDate`; tray title stays tied to today.

**Tech Stack:** Tauri (Rust), vanilla JS, HTML/CSS.

### Task 1: Storage helper to read a date without creating a file

**Files:**
- Modify: `src-tauri/src/storage.rs`
- Modify: `src-tauri/src/storage_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn read_items_for_date_does_not_create_file() {
    let root = tempdir().unwrap();
    let items = read_items_for_date(root.path(), "2026-01-01");
    let path = root.path().join("2026-01-01.md");

    assert!(items.is_empty());
    assert!(!path.exists());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test read_items_for_date_does_not_create_file` (from `src-tauri/`)
Expected: FAIL with "cannot find function" (or similar)

**Step 3: Write minimal implementation**

```rust
pub fn read_items_for_date(root: &Path, date: &str) -> Vec<String> {
    let filename = journal_filename(date);
    let path = root.join(filename);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    if text.trim().is_empty() {
        return Vec::new();
    }
    split_items(&text)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test read_items_for_date_does_not_create_file` (from `src-tauri/`)
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/storage.rs src-tauri/src/storage_tests.rs
git commit -m "feat: read items for date without creating file"
```

### Task 2: Day count listing helper for the strip

**Files:**
- Modify: `src-tauri/src/storage.rs`
- Modify: `src-tauri/src/storage_tests.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn list_day_counts_reads_existing_files() {
    let root = tempdir().unwrap();
    save_active_file(root.path(), "2026-01-01.md", "one\n\n---\n\ntwo\n");
    save_active_file(root.path(), "2026-01-02.md", "one\n");

    let counts = list_day_counts(root.path());
    assert!(counts.iter().any(|c| c.date == "2026-01-01" && c.count == 2));
    assert!(counts.iter().any(|c| c.date == "2026-01-02" && c.count == 1));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test list_day_counts_reads_existing_files` (from `src-tauri/`)
Expected: FAIL with "cannot find function" (or similar)

**Step 3: Write minimal implementation**

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DayCount {
    pub date: String,
    pub count: usize,
}

pub fn list_day_counts(root: &Path) -> Vec<DayCount> {
    let mut out = Vec::new();
    for name in list_markdown_files(root) {
        let date = name.trim_end_matches(".md").to_string();
        let path = root.join(&name);
        let text = std::fs::read_to_string(path).unwrap_or_default();
        out.push(DayCount {
            date,
            count: count_items(&text),
        });
    }
    out
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test list_day_counts_reads_existing_files` (from `src-tauri/`)
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/storage.rs src-tauri/src/storage_tests.rs
git commit -m "feat: add day counts listing for strip"
```

### Task 3: Backend commands for view date + counts

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/storage.rs`

**Step 1: Write the failing test**

Add a storage test to verify `read_items_for_date` output feeds a view struct (already covered), then wire command. No new Rust command test required.

**Step 2: Run test to verify it fails**

Run: `cargo test` (from `src-tauri/`)
Expected: PASS (no new failing test here)

**Step 3: Write minimal implementation**

```rust
#[derive(Serialize)]
struct DayItems {
    date: String,
    items: Vec<String>,
    count: usize,
}

#[tauri::command]
fn get_items_for_date(app: tauri::AppHandle, date: String) -> DayItems {
    let root = storage::storage_root();
    let items = storage::read_items_for_date(&root, &date);
    let count = items.len();
    set_window_title(&app, count);
    DayItems { date, items, count }
}

#[tauri::command]
fn list_day_counts(_app: tauri::AppHandle) -> Vec<storage::DayCount> {
    let root = storage::storage_root();
    storage::list_day_counts(&root)
}
```

- Ensure `append_today_item` keeps tray title updated but does not override the window title if we want it to reflect `viewDate` (front-end will update it when `viewDate == todayDate`).

**Step 4: Run test to verify it passes**

Run: `cargo test` (from `src-tauri/`)
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/storage.rs
git commit -m "feat: add view-date and day-count commands"
```

### Task 4: Day strip utilities + tests (frontend)

**Files:**
- Create: `src/day-strip.js`
- Create: `tests/day-strip.test.js`

**Step 1: Write the failing test**

```js
import assert from "node:assert/strict";
import test from "node:test";
import { formatViewDate, buildRecentDates, visibleDateCount } from "../src/day-strip.js";

test("formatViewDate formats YYYY-MM-DD", () => {
  assert.equal(formatViewDate("2026-01-07"), "Wed, Jan 7");
});

test("buildRecentDates returns descending dates", () => {
  const dates = buildRecentDates("2026-01-03", 3);
  assert.deepEqual(dates, ["2026-01-03", "2026-01-02", "2026-01-01"]);
});

test("visibleDateCount respects width", () => {
  assert.equal(visibleDateCount(200, 18, 6), 8);
});
```

**Step 2: Run test to verify it fails**

Run: `node --test tests/day-strip.test.js`
Expected: FAIL with "module not found" or missing exports

**Step 3: Write minimal implementation**

```js
const LABEL_FORMATTER = new Intl.DateTimeFormat("en-US", {
  weekday: "short",
  month: "short",
  day: "numeric",
});

export function formatViewDate(dateString) {
  const [y, m, d] = dateString.split("-").map(Number);
  const date = new Date(y, m - 1, d);
  return LABEL_FORMATTER.format(date);
}

export function buildRecentDates(todayString, count) {
  const [y, m, d] = todayString.split("-").map(Number);
  const date = new Date(y, m - 1, d);
  const out = [];
  for (let i = 0; i < count; i += 1) {
    const iso = date.toISOString().slice(0, 10);
    out.push(iso);
    date.setDate(date.getDate() - 1);
  }
  return out;
}

export function visibleDateCount(containerWidth, size, gap) {
  if (!containerWidth) return 0;
  return Math.max(1, Math.floor((containerWidth + gap) / (size + gap)));
}
```

**Step 4: Run test to verify it passes**

Run: `node --test tests/day-strip.test.js`
Expected: PASS

**Step 5: Commit**

```bash
git add src/day-strip.js tests/day-strip.test.js
git commit -m "feat: add day strip utilities"
```

### Task 5: Frontend UI wiring

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles.css`
- Modify: `src/main.js`

**Step 1: Write the failing test**

No DOM test harness exists; use manual verification after changes.

**Step 2: Run test to verify it fails**

Run: `node --test tests/day-strip.test.js`
Expected: PASS (baseline)

**Step 3: Write minimal implementation**

- Add day strip container and a “Back to Today” button in the drawer header.
- Render squares from `recentDates`, using counts map to display numbers.
- Switch `viewDate` on click and call `get_items_for_date` to refresh list/count.
- Keep editor writes targeting today via existing `append_today_item`.
- When `viewDate == todayDate`, update list and header from today items.
- Keep tray title tied to today by not calling any tray-setting logic from view-date calls.

**Step 4: Run test to verify it passes**

Run: `npm run dev` (manual):
- Verify squares appear and show counts.
- Clicking a square updates list and header label.
- Editor still writes to today.
- “Back to Today” returns to today and updates header/count.

**Step 5: Commit**

```bash
git add src/index.html src/styles.css src/main.js
git commit -m "feat: add day strip and view-date switching"
```

---

Plan complete and saved to `docs/plans/2026-01-07-multi-day-view-implementation-plan.md`.

Two execution options:

1. Subagent-Driven (this session) - I dispatch a fresh subagent per task, review between tasks
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
