# Minimal Markdown Formatting + Archive Toggle Design

## Intent
Add minimal visual formatting to the editor while preserving raw Markdown text and a distraction-free, monospace UI. Enable clickable checklist items that archive completed lines via the backend, with a discreet persistent toggle to show archived items inline and allow restore.

## Goals
- Preserve raw Markdown markers in the UI (e.g., show `**bold**` and `*italic*` literally).
- Provide visual emphasis for bold and italic without rich-text editing.
- Render checklist items as clickable controls that archive completed lines.
- Add a subtle, persistent bottom-center toggle to show archived items inline.
- Allow restoring archived items back into the active list.
- Keep the editor minimal, fast, and visually quiet.

## Non-goals
- Full Markdown rendering (headings, links, code, etc.).
- Rich text editing or contenteditable-based editing.
- Multi-pane or heavy UI additions.

## UX Behavior
- The editor remains a single full-height surface with a raw-text input.
- A display layer mirrors the text and adds minimal formatting:
  - `**bold**` appears bold while the literal `**` remains visible.
  - `*italic*` appears italic while the literal `*` remains visible.
- Checklist lines (`- [ ]` / `- [x]`) show a minimal checkbox UI aligned to the line.
- Clicking a checkbox temporarily strikes the line, then removes it from the active list.
- Completed items are moved to `## Archived` in the file by the backend.
- A discreet bottom-center toggle (e.g., "Show archived") reveals archived items inline.
- Archived items render with muted styling and a restore affordance.
- Restore moves the line back into active items and removes it from the archive.

## Architecture
- Use a layered renderer + hidden (or transparent) textarea:
  - `textarea` stays the source of truth for editing and cursor behavior.
  - A renderer layer builds read-only HTML from the textarea value.
  - Renderer enables pointer events only for checkboxes and restore controls.
- Renderer is synchronized on input and on load.
- Scroll position is synced between textarea and renderer.

## Data Flow
- On input: save schedule + re-render preview layer.
- On checkbox click:
  - UI sends `{ lineIndex, lineText }` to Rust command (e.g., `archive_item`).
  - Backend removes the line from active section and appends it under `## Archived`.
  - UI updates (either re-fetch or local update + re-render).
- On "Show archived": renderer shows active + archive sections inline.
- On restore click:
  - UI sends `{ lineIndex, lineText }` to Rust command (e.g., `restore_item`).
  - Backend removes the line from archive and inserts it back into active items.
  - UI updates and re-renders.

## Edge Cases and Handling
- If line index is stale, backend verifies `lineText` before moving.
- If mismatch, backend returns error and UI re-fetches current file.
- If `## Archived` header is missing, backend creates it once.
- If `## Archived` is not at the end, backend appends under the last archive block.
- Archived section is view-only; editing still happens in textarea.

## Testing
- Rust unit tests for:
  - Archive when no `## Archived` exists.
  - Archive when `## Archived` exists.
  - Restore from archive to active list.
  - Mismatch handling for stale index.
- Manual verification for UI:
  - Typing still feels native and fast.
  - Bold/italic render while markers remain visible.
  - Checklist click archives and removes from active view.
  - Toggle shows/hides archive inline.
  - Restore returns item to active list.

## Open Questions
- Exact label and visual styling for the archive toggle.
- Whether to re-fetch file on each archive/restore or apply local updates.
