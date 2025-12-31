# Inbox App Tauri MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a minimal Tauri macOS app that captures notes in journal mode with editable items, archiving on check, and counts across files.

**Architecture:** Tauri (Rust backend + vanilla HTML/CSS/TS frontend). Rust owns storage, counting, and file mutation; the UI is a single-pane editor with a list and counts, and communicates via Tauri commands.

**Tech Stack:** Tauri, Rust, TypeScript, Vite (vanilla), macOS.

**Editing Guardrail:** Free edit any line in the active (unprocessed) list.

**Key Design Decisions:**
- Journal-only for v1: use `YYYY-MM-DD.md`. Clean mode deferred.
- On open, show today’s file with cursor at the end.
- Local storage root: `~/.inboxapp`.
- Checking an item moves it into a hidden `## Archived` section in the same file (not rendered in UI).
- Counts: current file items + total items across all files + total file count (all shown).
- Menubar + docked window; menubar toggles the window and shows counts.

### Task 1: Scaffold Tauri + Vite (vanilla TS)

**Files:**
- Create: `package.json`
- Create: `src/` (Tauri frontend)
- Create: `src-tauri/` (Tauri backend)

**Step 1: Create the app skeleton**

Run: `npm create tauri-app@latest . -- --template vanilla --title InboxApp --bundle-identifier com.inboxapp.desktop --manager npm`

Expected: A Tauri + Vite skeleton with `src/` and `src-tauri/` created.

**Step 2: Verify dev server starts**

Run: `npm install`

Run: `npm run tauri dev`

Expected: App window opens with default template.

**Step 3: Commit**

```bash
git add package.json src src-tauri

git commit -m "chore: scaffold tauri app"
```

### Task 2: Implement storage model in Rust

**Files:**
- Create: `src-tauri/src/storage.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/src/storage_tests.rs`

**Step 1: Write failing tests for storage helpers**

```rust
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
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -q`

Expected: FAIL (helpers not implemented).

**Step 3: Implement storage helpers**

Implement:
- `storage_root() -> PathBuf` (uses `~/.inboxapp`)
- `journal_filename(date: &str) -> String`
- `split_archived(text: &str) -> (Vec<String>, Vec<String>)`
- `archive_line(text: &str, line_idx: usize) -> String`
- `count_items(text: &str) -> usize`
- `list_markdown_files(root) -> Vec<String>`

**Step 4: Run tests to verify they pass**

Run: `cargo test -q`

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/storage.rs src-tauri/src/storage_tests.rs src-tauri/src/main.rs

git commit -m "feat: add storage helpers"
```

### Task 3: Add Tauri commands for load/save/archive/count

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/storage.rs`
- Test: `src-tauri/src/storage_tests.rs`

**Step 1: Write failing tests for file IO**

```rust
#[test]
fn load_creates_file_if_missing() {
    let root = tempdir::TempDir::new("inboxapp").unwrap();
    let content = load_or_create(&root.path().into(), "2025-12-31.md");
    assert!(content.is_empty());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -q`

Expected: FAIL.

**Step 3: Implement file IO and Tauri commands**

Add commands:
- `get_active_file() -> { filename, text, counts }`
- `save_active_file(filename, text) -> counts`
- `archive_item(filename, line_idx) -> { text, counts }`
- `list_files() -> { files, counts }`

**Step 4: Run tests to verify they pass**

Run: `cargo test -q`

Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/main.rs src-tauri/src/storage.rs src-tauri/src/storage_tests.rs

git commit -m "feat: add tauri commands"
```

### Task 4: Build minimal UI (editor + list + counts)

**Files:**
- Modify: `src/index.html`
- Modify: `src/main.ts`
- Modify: `src/style.css`

**Step 1: Write basic HTML layout**

```html
<div class="app">
  <header>
    <div class="title">Inbox</div>
    <div class="counts" id="counts"></div>
    <div class="mode">Journal</div>
  </header>
  <main>
    <textarea id="editor"></textarea>
  </main>
  <section id="list"></section>
</div>
```

**Step 2: Implement UI behavior in TS**

- Call `get_active_file` on load and render list + textarea.
- Parse active lines from textarea into list with checkboxes.
- On checkbox click, call `archive_item` with line index.
- On textarea input, save to backend via `save_active_file` (debounced).
- Keep cursor at end on load.

**Step 3: Add minimal, quiet styling**

- Single column, subtle counts, large editor font.

**Step 4: Manual verify**

Run: `npm run tauri dev`

Expected: You can type, see list items, check to archive, counts update.

**Step 5: Commit**

```bash
git add src/index.html src/main.ts src/style.css

git commit -m "feat: build minimal ui"
```

### Task 5: Menubar toggle + docked window

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/main.rs`

**Step 1: Configure menu bar**

- Add tray icon and toggle window command.

**Step 2: Manual verify**

Run: `npm run tauri dev`

Expected: Menubar icon toggles the window.

**Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/src/main.rs

git commit -m "feat: add menubar toggle"
```
