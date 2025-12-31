# InboxApp Minimal Editor Design

## Goal
- Refactor the Tauri app UI into a black-and-white, iA Writer-inspired minimalist editor.
- Remove all preview/list UI and focus on fast, distraction-free typing.
- Show only today's count in the native window title as `Inbox — N`.

## Visual Design
- Single-page, blank-sheet feel with no visible chrome beyond the editor.
- Centered text column with generous left/right gutters (max width ~680-720px).
- Background: barely warm off-white (#faf9f6).
- Text: pure black; no shadows, cards, or gradients.
- Typography: SF Mono at 16-18px with line height ~1.6.
- Placeholder: subtle, minimal, low-contrast; disappears on input.
- Focus/selection: subdued, minimal visual treatment.

## Architecture and Components
- One full-height editor element; no header, no list, no secondary panels.
- Root wrapper fills the viewport and centers the text column.
- Title bar count: update native window title to `Inbox — N`.
- Keep DOM minimal to reduce layout and render overhead.

## Data Flow and Behavior
- Startup: load today's file, set editor content, move cursor to end.
- Autosave: debounce (300-500ms) and persist raw Markdown as typed.
- Counting: compute today's count from active file and update window title.
- No rendering layer: remove list rendering and archive interactions from UI.
- Minimal events: input-driven save only; avoid extra DOM work.

## Error Handling
- Log errors to console; no in-UI alerts or banners.
- Save failure: keep text in editor; retry on next debounce.
- Load failure: fall back to empty editor and allow typing/saving.
- Title update failure: ignore and continue (non-critical).

## Testing and Verification
- Smoke test: launch app, type, autosave, title shows `Inbox — N`.
- Confirm no list/preview UI is rendered.
- Ensure counts logic still correct for today's file.

## Non-Goals
- Inline Markdown rendering (raw Markdown only).
- Rich editing, checklists, or additional UI features.
- Multi-file navigation in the UI.
