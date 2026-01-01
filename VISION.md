# Inbox App (working title)

Capture without thinking. Process later.

A hyper-minimal macOS inbox app for capturing fleeting thoughts in Markdown with near-zero friction, designed explicitly to be emptied.

## Product brief

### Overview

Inbox Zero is a lightweight, local macOS desktop app whose sole purpose is to capture ideas, notes, and reminders the moment they arise, without disrupting the user’s cognitive flow. It functions as a transient inbox, not a long-term store: everything captured is intended to be processed and moved elsewhere.

The app deliberately excludes intelligence, organization, and structure at capture time. Its success metric is not accumulation, but clearance: reaching inbox zero.

## Vision

Create the fastest possible path from thought to text, minimizing attention switching, decision-making, and visual distraction. Inbox Zero is designed to sit orthogonally to knowledge systems like Obsidian: it absorbs raw input so that sense-making can happen later, deliberately and elsewhere.

## Core purpose

* Minimize the cognitive and temporal cost of capture.
* Preserve deep focus by avoiding navigation, context switching, and incidental stimuli.
* Provide a psychologically clear, finite inbox that encourages regular processing to zero.

## Functional principles

* Single responsibility: capture only.
* Zero configuration at use time.
* One surface, one action: write.
* Markdown-first, Obsidian-compatible output.
* Local, offline, and fast.
* No AI, no summarization, no restructuring.
* Explicitly non-archival.

## Core functionality

### Capture

* On open, show today’s capture buffer (or a blank buffer scoped to today).
* Append-only input.
* Each entry automatically timestamped.
* Default entry style: bullet point or newline-separated plain text.
* Accepts typed, pasted, or dictated input identically.

### Inbox semantics

* Each capture is an “unprocessed item.”
* The app displays a global count of unprocessed items.
* The explicit goal is to reduce this count to zero.
* Items are removed from the inbox only when the user intentionally moves or clears them.

### Processing (minimal)

* Ability to mark items as processed by removing or exporting them.
* No internal categorization, tagging, or hierarchy.
* Processing is expected to happen into external systems (e.g. Obsidian vaults).

## Job stories

* When an idea occurs while I am focused on another task, I want to capture it in under a second so that I can immediately return to my flow.
* When I open the app, I want to see a single writing surface so that I never have to navigate or decide.
* When I add a note, I want it timestamped automatically so that I don’t think about structure.
* When I glance at the app, I want to see how many unprocessed items remain so that I feel a clear pull toward inbox zero.
* When I process my inbox, I want everything to leave the app so that it never becomes a secondary knowledge base.
* When I use dictation, I want the app to behave exactly like a plain text editor.

## Non-goals

* Long-term storage.
* Knowledge management.
* Task management.
* Smart suggestions or automation.
* Cross-device sync (initially).

## Implementation notes

* Platform: native macOS desktop app.
* Form factor: always-available, instant-switch application (keyboard shortcut optional, not required).
* May want to use Tauri or similar (want ultra-lightweight)
* Editor: ultra-lightweight Markdown / plain-text editor.
  * Initial implementation may be raw Markdown only.
  * Optional future enhancement: rendered Markdown view toggle.
* Data model:
  * Daily files or buffers.
  * Simple append log with timestamps.
  * Persistent count of unprocessed items.
* Performance constraints:
  * Minimal memory footprint.
  * Near-instant launch and focus.
* Storage:
  * Local files, human-readable.
  * Compatible with downstream Markdown tools (e.g. Obsidian).

## Design ethos

Inbox Zero should feel closer to a notepad than an application: visually quiet, cognitively empty, and reliably fast. Its value lies not in what it does, but in what it refuses to do.


---
---


# Simplication and Clarification: simple and minimalist 2026-01-01

Should the app function as a continuous notepad, or as a single blank capture surface with enforced commit semantics?

# Core constraint (restated)

The app must not become:

* a to-do manager,
* a secondary notebook,
* or a place where attention drifts to previously captured items.

Its sole function is to minimize capture friction and defer all meaning-making.

# Option A: Continuous notepad (today’s page)

Description
The app opens to a running, append-only text buffer for the day. New entries are added below previous ones.

Advantages

* Trivial implementation.
* Familiar mental model.
* Easy to review and copy later.

Failure modes

* Visual exposure to prior entries creates distraction.
* Encourages micro-editing and rereading.
* Gradually becomes “just another notes file.”
* Weakens the psychological separation between capture and processing.

Assessment
This option optimizes convenience at the expense of the core intent. Over time, it will almost certainly drift toward light knowledge management.

# Option B: Blank capture surface with commit

Description
The app always opens to a blank input area. On commit (e.g. focus loss, shortcut, or explicit action), the content is sealed and moved out of view into an internal inbox list.

Advantages

* Zero visual distraction at capture time.
* Strong cognitive boundary: capture is ephemeral and irreversible.
* Enforces forward motion toward processing.
* Makes it structurally impossible to “tidy up” instead of thinking.

Risks

* Reduced ability to amend or correct entries.
* Requires a clear commit gesture to avoid confusion.
* Slightly higher implementation complexity.

Assessment
This option is far more aligned with the stated purpose. The loss of editability is a feature, not a bug.

# Recommendation

Adopt Option B: a blank, single-purpose capture surface with enforced commit and no inline editing of committed items.

This preserves the app as an instrument, not a workspace.

# Clarifying the editing question

Your instinct to disallow editing of committed items is sound. Editing encourages premature sense-making and undermines flow preservation.

Recommended rule:

* Before commit: free editing.
* After commit: no editing, only disposition.

If correction is needed, the correct action is re-capture, not revise.

# Minimal processing model (v1)

To support inbox zero without becoming a task manager:

States (implicit, not visible as a list)

* Unprocessed (default on commit)
* Cleared (no longer counted)

Actions

* Export / copy (moved elsewhere → cleared)
* Delete (irrelevant → cleared)

Not included (by design)

* Scheduling
* Tagging
* Prioritization
* Status beyond “still here” vs “gone”

The inbox count reflects only unprocessed items.

# UI structure (conceptual)

* Primary surface:

  * Always blank on focus.
  * Plain text / raw Markdown.
  * Cursor ready immediately.

* Secondary surface (optional, non-prominent):

  * Hidden or collapsed list of committed items.
  * Only visible during deliberate processing moments.
  * Never shown during capture by default.

# Purpose clarity (final test)

If the app ever tempts you to:

* reread,
* tweak wording,
* or reorganize,

then it is failing.

If it feels slightly austere and mildly unforgiving, it is likely succeeding.

This design keeps the app sharply differentiated from Obsidian, Drafts, and GTD tools, and maximally faithful to your stated aim: capture fast, think later, empty completely.

