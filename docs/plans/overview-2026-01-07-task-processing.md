NB: decided this was overelaborate and went for something much simpler.

- Goals: support multiple task outcomes beyond “done”; keep items visible with clear resolution icons; minimize irreversible actions.
- Statuses: “moved” (renamed from “processed”), “done”, and “won’t fix”; each represented by an emoji placed in the left status slot.
- Emojis: 🚚 for moved, ✅ for done, ❌ for won’t fix (red X).
- Visibility: items remain visible after resolution; “moved”/“won’t fix”/“done” items are greyed out; only trashed items are hidden.
- Delete behavior: not destructive; remove from list but move to a rolling trash.md with timestamp so it’s recoverable.
- Actions: per‑item inline icons on hover (no overflow menu, no extra clicks); icons themselves are the emojis.
- Copy behavior: clicking Copy should also set “moved” status; moved can also be set directly via the 🚚 action icon.
- File storage: status is written into the markdown line itself (emoji inserted after any leading checkbox, e.g., [ ] 🚚 …), not app‑only state.
- UI placement: status emoji shown on the left, adjacent to the checklist indicator; in the complex proposal we debated whether the emoji replaces the checkbox or sits next to it.
