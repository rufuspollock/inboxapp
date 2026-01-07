# Simple Task Checkbox + Hover Delete Design

**Goal:** Keep task completion minimal by adding a single checkbox and a hover-only delete action while preserving Markdown compatibility.

**Context:** The drawer list shows today/view date items. We want a clean UI with minimal controls and reversible deletion.

## UX Summary

- A checkbox is always visible on the left of each task row.
- Clicking the checkbox marks the item as done and greys out the row.
- No extra statuses or emoji markers in this iteration.
- Copy and Delete are hidden until hover/focus; Delete appears next to Copy.
- Deleted items disappear from the list and are appended to a rolling `trash.md` file.

## Data Model + Storage

- Tasks are stored as Markdown lines in daily files (e.g., `YYYY-MM-DD.md`).
- Done tasks use standard task list syntax: `- [x] <text>`.
- Open tasks use `- [ ] <text>`.
- Existing lines without task markers are treated as unchecked in the UI; the file is only normalized when the user modifies a line (check/uncheck/delete).
- Deletions remove the line from the daily file and append a timestamped entry to `trash.md` (single rolling file), ensuring reversibility.

## UI/Component Behavior

- Each list item renders a checkbox, task text, and (on hover) action buttons.
- Checking an item updates the underlying file line to `- [x] <text>` and greys the row.
- Unchecking (if supported) restores `- [ ] <text>`.
- Copy keeps Markdown task markers intact so downstream paste keeps the list structure.
- Delete action removes the item from the list and persists the deletion to `trash.md`.

## Error Handling

- If file updates fail, show a transient drawer status message and keep UI state unchanged.
- Clipboard failures continue to show "Copy failed" without blocking other actions.

## Testing Notes

- Add unit coverage for parsing task lines into display state (checked/unchecked).
- Add unit coverage for updating a line to `- [x]` and for removing a line.
- Add integration coverage that delete appends to `trash.md` with timestamp and removes from daily file.

## Non-Goals

- Multiple completion states (moved/ignored/emoji).
- Hidden completed tasks.
- Bulk operations beyond existing "copy all".
