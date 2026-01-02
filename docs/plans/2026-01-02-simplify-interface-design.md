# Simplify Interface Design

## Goal
- Deliver a cognitively irreversible capture surface that is always blank on focus and commits on app blur.
- Keep review secondary via a collapsible “Today” drawer showing today’s items only.

## Architecture
- Single-page UI with two stacked zones: primary capture textarea and secondary “Today” drawer.
- Capture commits to a Markdown file on blur; review reads from the same file.
- No editing of committed items in the capture surface.

## Core Behaviors
- On window blur: if textarea contains non-whitespace text, append it to today’s Markdown file and clear the textarea.
- On window focus: force-clear textarea so capture is always blank.
- Drawer starts collapsed; click/tap header toggles open/closed and stays in that state.

## Data Flow & Persistence
- Storage: one Markdown file per day (e.g., `YYYY-MM-DD.md`), append-only.
- Item separation: use a simple, consistent Markdown divider (e.g., `###` heading or `---`); no timestamps for now.
- Load: parse today’s file into an in-memory list; append new items immediately on blur commit.

## UI Layout & Interactions
- Capture area: large, quiet, dominant; focused on load and focus gain.
- Drawer header: shows “Today” and a count badge; entire header is the toggle target.
- Drawer list: muted colors, single-line truncation with ellipsis; subtle separators.
- Each item row shows a muted checkbox icon on the left (purely visual, no state yet).
- Copy actions: per-item copy control and a “Copy all” control for today’s items.

## Error Handling
- If append fails: log error, keep text in textarea, show subtle warning in drawer header.
- If today’s file is missing: treat as empty; if partially malformed, parse best-effort entries.

## Testing & Verification
- Unit tests: trim/ignore empty behavior, markdown append/parse, persistence write/restore logic.
- UI tests: blur commit, focus clear, drawer toggle, per-item copy, copy-all.
- Manual checks: blur commit via app focus change; confirm blank capture on refocus; verify drawer toggle persistence.

