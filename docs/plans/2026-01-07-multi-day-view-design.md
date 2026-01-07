# Multi-day view design

## Goal
Enable viewing previous days with minimal UI change while keeping capture flow fast and unchanged.

## Core UX decisions
- Show a single day at a time in the drawer/list.
- Editor always writes to today, even when viewing past days.
- Drawer header shows the active view date, e.g., `Tue, Mar 5 — N`.
- Add a bottom day strip with small square chips that fit across the window.
- Each square shows the item count inside; 0 uses a neutral/gray style.
- Color intensity reflects count (muted green scale); active day has a clear selected outline.
- “Back to Today” affordance only appears when a past day is selected.

## Data + behavior
- `todayDate` is the write target; `viewDate` controls the list/header.
- Day strip shows a continuous range of recent dates (last N that fit), even if no file exists.
- Empty days do not create files until a write happens.
- Switching squares updates `viewDate`, reloads items for that date, and keeps editor focused.
- Window title count uses the active view date; tray title stays tied to today’s count.

## Components
- Day strip component renders `recentDates` and handles selection.
- Header uses a formatted date label for `viewDate`.
- Drawer list renders items for `viewDate` and supports copy actions as today.

## Backend/API needs
- New command: fetch items for a given date (read-only).
- Existing `list_files` used for counts per day in the strip.

## Error handling
- If loading a past day fails, show `Load failed` and keep current view.
- If listing files fails, fall back to a strip that only shows today.

## Testing
- Storage tests for loading items by date.
- Date parsing/formatting helper tests for `viewDate` display.

## Future considerations
- Optional: allow editing past days (would require writing to `viewDate`).
- Optional: tabbed history view.
