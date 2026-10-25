# Atomic entries and image capture

Approved direction: individual portable Markdown entries replace daily journals;
image capture follows as a separate change. Execution is tracked in Beads epic
`inboxapp-5b5`, with image capture dependent on the storage migration.

## Storage contract

Each entry has an immutable filename ID including its date. The filename is the
source of its date; the body remains ordinary Markdown. `config.yml` records the
storage schema version independently of the app version. Existing daily journals
are backed up before migration; interrupted migration must resume without
duplicating entries or losing the original files. Unsupported future versions
fail explicitly. Reading failures must never appear as an empty inbox.

Edits and deletions target IDs and check the original content, so externally
moving or editing files cannot redirect an operation to another entry. The day
picker is a view over individual entries, not a storage boundary. Deletion keeps
the entry and its owned attachments recoverable in Trash.

## Image ownership and UX

Each entry owns `assets/<entry-id>/`. No cross-entry deduplication: removing one
entry must not damage another. Import stores validated image bytes durably before
inserting a relative Markdown link. Clipboard screenshots and an image picker use
the same path. Plain text paste stays plain text. Modest previews accompany the
Markdown editor and saved entries; image-only captures are valid.

Capture drafts retain their ID and attachment links across restarts. Save remains
explicit. Pending imports prevent conflicting actions; failures retain the draft
and never insert a broken link. Discarding a capture draft removes only its owned,
unsaved attachments. Cancelling an edit must preserve saved entry attachments.
Copy remains Markdown text and does not transfer image bytes to other apps.

## Validation and delivery

Rust tests exercise migration/restart, filename collisions, ID-targeted mutations,
conflicts, filesystem errors, and attachment ownership. Browser-level DOM tests
exercise capture/edit/paste/recovery and error states. Run both suites and a Tauri
build before handoff. Feature changes get convention-compliant changelog snippets
and appear on the website. Commit storage and image work separately, push the
repository, and sync Beads through Dolt. Release automation remains the existing
release-please flow; cutting a release is separate from shipping these changes.
