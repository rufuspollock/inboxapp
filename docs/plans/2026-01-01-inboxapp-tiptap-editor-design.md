# InboxApp TipTap Editor Design (Superseded)

## Motivation
- Current textarea-based editor has usability issues (notably copy/paste and dictation).
- A more robust editor stack is needed to support minimal styling now and future
  features like checkboxes and archive interactions.
- We want to keep data interoperable with other Markdown apps (e.g., Obsidian).

## Status
- Design only. Not implemented.
- This direction is paused in favor of an even more minimal approach to be
  defined separately.

## Goal
- Replace the textarea with a minimal TipTap/ProseMirror editor.
- Preserve the existing minimalist UI (off-white page, centered column).
- Keep storage as raw Markdown for compatibility.
- Ensure copy/paste works reliably and copies as Markdown.

## Recommended Approach
- Use TipTap (ProseMirror-based) with a minimal schema and a Markdown
  serialization pipeline.
- Keep editor state as ProseMirror; serialize to Markdown on save.

## Architecture and Components
- Introduce a React entry point and mount a single editor component.
- HTML reduces to a single `#root` container.
- Render only the editor surface (no toolbar, no chrome).
- Use a minimal schema: Document, Paragraph, Text, HardBreak, History.
- Add TaskList/TaskItem for checkbox support (future interactions).
- Apply current minimal styling to `.ProseMirror` instead of `textarea`.

## Data Flow and Behavior
- Startup: `get_active_file` returns filename + Markdown.
- Parse Markdown into ProseMirror doc as initial editor content.
- Focus editor on load and move cursor to end.
- Autosave: debounce (400-500ms) and call `save_active_file`.
- Serialize editor state to Markdown on save.
- Title updates remain in Rust (`save_active_file` and `get_active_file`).

## Clipboard and Interop
- Copy: ensure `text/plain` contains Markdown for external pastes.
- Paste: accept plain Markdown and HTML; normalize to Markdown before parse.
- Round-trip Markdown to preserve compatibility with other apps.

## Checkbox Behavior (Future)
- Use TaskList/TaskItem to toggle checkboxes inline.
- Keep Markdown `- [ ]` / `- [x]` format on save.
- Optional archive-on-check can be implemented by moving checked items into an
  archive section in Markdown.

## Risks and Trade-offs
- Adds a full editor stack and React to a previously vanilla UI.
- Requires Markdown <-> ProseMirror conversion logic.
- Clipboard handling needs deliberate configuration.

## Non-Goals
- Rich formatting UI (bold/italic/toolbars).
- WYSIWYG presentation beyond minimal styling.
- Inline Markdown rendering beyond what TipTap provides by default.
