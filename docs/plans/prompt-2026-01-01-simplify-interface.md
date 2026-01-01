# Pared Down Interface

The core insight is that capture should be cognitively irreversible and visually empty. The moment of capture must not expose the user to previous thoughts, decisions, or unfinished items, because any such exposure competes with the primary task the user is returning to. Therefore, capture is separated structurally and temporally from review. The app enforces a strict asymmetry: capture is immediate, blank, and forgiving; review is deliberate, bounded, and secondary.

The application functions as a transient inbox, not a notebook. On focus, the user is presented with a single blank surface into which they can type or dictate freely. On commit (e.g. explicit action, focus loss, or shortcut), the content is sealed, timestamped, and logged in the background. The capture surface is immediately cleared. Editing of committed items is intentionally disallowed to prevent micro-sense-making during capture.

All captured items are persisted locally, most likely as Markdown, with simple metadata (timestamp, date, unique id). Storage may be organized per day or append-only; the implementation detail is secondary to the invariant that capture never requires navigation or file choice. A lightweight secondary view allows the user, at a later time, to review items (e.g. today, another day, or all), process them out of the app, and reduce the unprocessed count toward inbox zero.

---

## High-level functional shape

* Primary mode: capture

  * Always blank on entry.
  * Plain text / Markdown input.
  * One action: write.
  * One transition: commit → clear.

* Secondary mode: review / process

  * Explicitly entered.
  * Read-only list of committed items.
  * Minimal actions: copy/export, delete/clear.
  * Displays count of unprocessed items.

The primary mode is optimized for speed and invisibility; the secondary mode is optimized for closure.

---

## Conceptual ASCII sketch

```
┌──────────────────────────────────────────────┐
│ Inbox Zero                              (3) │
├──────────────────────────────────────────────┤
│                                              │
│  Capture                                      │
│  ──────────────────────────────────────────  │
│                                              │
│  > _                                          │
│                                              │
│  (type or dictate freely)                     │
│                                              │
│                                              │
├──────────────────────────────────────────────┤
│  Today (collapsed by default)                 │
│  ──────────────────────────────────────────  │
│  • 09:14 — Idea about X                       │
│  • 11:02 — Note on Y                          │
│  • 16:47 — Reminder re Z                      │
│                                              │
│  [Copy] [Delete]                              │
└──────────────────────────────────────────────┘
```

Notes:

* The top capture area is always blank on focus.
* The lower section is optional, secondary, and visually subordinate.
* The number in the header reflects unprocessed items.
* No item appears in the capture area once committed.

This framing keeps the app sharply defined as an instrument for capture and clearance, not reflection or organization.

