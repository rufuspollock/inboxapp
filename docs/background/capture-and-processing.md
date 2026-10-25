# Capture now, process later

InboxApp supports a broader personal workflow: capture a thought or reference immediately, then decide where it belongs later. This background brings the useful rationale from the separate `inbox` experiment into the app project. It is context for product decisions, writing and sharing, rather than a specification of features the app already provides.

## Why an inbox?

While reading H.M. Bateman cartoons, Rufus noticed how vividly they demonstrated cultural norms in England. He wanted to photograph a page and capture a sentence about cultural evolution. The friction was deciding where it belonged: a website log, a knowledge base, project notes, or somewhere else. The routing decision interrupted the thought.

The same problem occurs with a dictated idea, a bookmark, a screenshot, a quotation or a longer note. Capture should preserve the material and the brief thought that made it worth saving, without requiring a destination or category first.

The principles are simple:

- Capture quickly through whichever input is at hand: desktop, dictation, phone camera or chat.
- Keep the content in accessible Markdown and files, so it can leave the capture tool easily.
- Separate capture from processing. Review later, route useful material to its permanent home, and clear the inbox.
- Keep tasks in the task system, references in knowledge bases, and writing in its appropriate project. The inbox is a temporary holding place.

## Where InboxApp fits

InboxApp provides the quiet desktop capture surface. Its current implementation stores each capture in its own dated Markdown file in `~/.inboxapp`, with entry-owned image attachments and previews. This brings it closer to the old experiment's file-based capture model, although it does not implement that experiment's Telegram-to-Git pipeline.

The broader workflow overlaps bookmarking: a saved link becomes more useful when its source, context and the reason for saving it survive processing. In Rufus's personal planning, **Papertrail** is the umbrella for capture, bookmarking, knowledge-base building and AI-assisted routing. InboxApp is a capture component in that picture; those broader ambitions do not automatically become product commitments.

The September 2026 planning notes report that desktop capture worked, but manual processing was unreliable, and that phone capture had moved from OpenClaw to Hermes via Telegram. The lesson is that a fast input surface needs a dependable route out of the inbox, even when processing happens outside the app.

## Origin and consolidation

InboxApp began on 31 December 2025 (`949abfc`). The separate `rufuspollock/inbox` repository began on 30 March 2026 (`1968b16`) with a README, motivation, agent instructions and empty capture/asset folders. Its proposal was Telegram via OpenClaw into one Markdown file per capture. It had an independent Git history, not a branch of InboxApp.

On 25 October 2026, Rufus chose to consolidate the inbox work and rationale into InboxApp. Since the initial comparison, the app has gained one-file-per-entry storage (`433c1f7`) and image attachments (`c135029`). This documentation consolidation preserves the useful background without merging unrelated Git histories or making further storage changes. Rufus subsequently approved deleting the redundant repository. The old local checkout and GitHub repository have both been deleted; retirement is complete (`inboxapp-ioc`).

Source: the former `rufuspollock/inbox` repository's `MOTIVATION.md`, initial commit `1968b16a6bdcbe81cd69a97472812174416cee33`. Its useful rationale is preserved above. Personal workflow context remains in the planning repository's `projects/ideas-inbox-capture-system.md` and `initiatives/papertrail.md`.
