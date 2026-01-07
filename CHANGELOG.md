### 2026-01-07 ✨

- Add a bottom swim-lane day strip that surfaces recent days as tiny squares with count-based color intensity.
- Clicking a day square switches the Today drawer to that day while keeping the capture editor writing to today.
- Show “Today” label for the active day, show dates for past days, and include a Back to Today affordance.
- Empty days appear as gray squares with no count, and the strip stops once there are no earlier non-empty days.

### 2026-01-06

- Fix stale daily file writes when the app stays open past midnight; appends now always target today's file.
- Show newest Today items at the top of the list.
- Lock window scrolling and hide scrollbars by scrolling within the editor and Today panel.
- Tighten the Today list rows with compact spacing and subtle separators.

### 2026-01-02 — Simplify Interface

- Blank-on-focus capture surface with blur-to-commit flow and Markdown append-only storage per day.
- Collapsible Today drawer with muted list view and copy actions for single items or all.
- Reliable Cmd+V paste on macOS via native clipboard integration.
