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

