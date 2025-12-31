# Minimal Editor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor the UI into a single, iA Writer-inspired minimalist Markdown editor and show today's count in the native window title as `Inbox — N`.

**Architecture:** Keep the existing Tauri storage and counting logic. Simplify the frontend to a single textarea and update Rust commands to set the main window title using the current count on load and on save.

**Tech Stack:** Tauri (Rust), vanilla HTML/CSS/JS.

### Task 1: Update Rust title formatting and window title updates

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/lib.rs` (existing unit tests)

**Step 1: Write the failing test**

Add a new unit test to assert the formatted window title:

```rust
#[test]
fn formats_window_title_with_count() {
    assert_eq!(format_window_title(7), "Inbox — 7");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test tray_title_tests::formats_window_title_with_count`
Expected: FAIL with "cannot find function `format_window_title`" or wrong output.

**Step 3: Write minimal implementation**

Implement `format_window_title(count: usize) -> String` and update command flow to call `set_window_title(&app, counts.current)` in `get_active_file`, `save_active_file`, and `archive_item`.

**Step 4: Run test to verify it passes**

Run: `cargo test tray_title_tests::formats_window_title_with_count`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: update window title with today count"
```

### Task 2: Simplify HTML structure to editor-only view

**Files:**
- Modify: `src/index.html`

**Step 1: Update the markup**

Replace the header/list structure with a single editor wrapper:

```html
<div class="app">
  <textarea id="editor" class="app__editor" placeholder="Type..." spellcheck="true"></textarea>
</div>
```

**Step 2: Smoke check in dev build**

Run: `npm run tauri dev`
Expected: Only the textarea is visible.

**Step 3: Commit**

```bash
git add src/index.html
git commit -m "feat: simplify editor layout"
```

### Task 3: Apply minimalist iA Writer-inspired styling

**Files:**
- Modify: `src/styles.css`

**Step 1: Update the base theme**

Set background to #faf9f6, use SF Mono, remove borders/shadows/cards, and center a max-width column (680-720px). Use a full-height layout with generous padding.

**Step 2: Run the app to verify visual direction**

Run: `npm run tauri dev`
Expected: Centered column, blank-sheet feel, no visible chrome.

**Step 3: Commit**

```bash
git add src/styles.css
git commit -m "feat: apply minimal editor styling"
```

### Task 4: Remove list rendering and counts from the frontend

**Files:**
- Modify: `src/main.js`

**Step 1: Remove list + counts wiring**

Delete `countsEl`, `listEl`, `renderList`, and `updateCounts` usage. Keep `splitArchived`/`combineWithArchived` only if still needed for storage structure.

**Step 2: Keep autosave and cursor behavior**

Ensure `boot()` loads today, sets editor content, and moves cursor to end. `input` should only schedule save.

**Step 3: Smoke check**

Run: `npm run tauri dev`
Expected: Typing works, autosave runs, no list UI.

**Step 4: Commit**

```bash
git add src/main.js
git commit -m "feat: remove list and counts from ui"
```

### Task 5: Final verification

**Files:**
- Verify: `src/index.html`, `src/styles.css`, `src/main.js`, `src-tauri/src/lib.rs`

**Step 1: Run app and verify title**

Run: `npm run tauri dev`
Expected: Window title reads `Inbox — N` and updates after saves.

**Step 2: Run Rust tests**

Run: `cargo test tray_title_tests::formats_window_title_with_count`
Expected: PASS.

**Step 3: Commit any final tweaks**

```bash
git add src src-tauri/src/lib.rs
git commit -m "chore: verify minimal editor refactor"
```
