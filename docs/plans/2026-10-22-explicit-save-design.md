# Explicit saving and editing

Approved by Rufus on 2026-10-22 for inboxapp-ssc.

Capture text remains in the editor when the app loses or gains focus. Save and Command+Enter append the text to today's Markdown journal and clear the editor only after a successful write. An unsaved draft is restored after restarting the app. Empty drafts cannot be saved; failures retain the text and show a message.

Clicking the text of any saved entry opens a multiline editor in its row with Save and Cancel controls. Saving replaces the entry in its original day, preserves its checked state, and leaves other entries untouched. Cancel leaves the saved entry unchanged. Date navigation must not silently discard an edit. Existing Markdown storage, copying, checkboxes, and trash behavior continue to work.

Implementation uses the existing Tauri append/update commands, browser draft storage, and inline row controls. Verification covers blur/focus, restart recovery, explicit saves, failed saves, duplicate submission, past-day editing, cancel, and preserving checked and multiline content. A separate Bead covers marking items as processed and recording their destination.
