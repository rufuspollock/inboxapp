# Testing Inbox on macOS

Use all three layers when changing capture, paste, or attachments. Passing the
first two does not prove the macOS clipboard or installed application works.

## Automated suites

```sh
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

The JavaScript suite uses JSDOM to exercise the editor, selection, paste events,
draft recovery, and errors. It mocks Tauri commands; it cannot validate native
clipboard formats, permissions, or IPC wiring. Rust tests use temporary storage
folders to exercise image encoding, attachment ownership, migration, durable
writes, and deletion/recovery. They do not drive the application window.

## Native reproduction

First identify the installed version and the actual running executable:

```sh
/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' /Applications/Inbox.app/Contents/Info.plist
ps -axo pid,command | rg '[I]nbox.app/Contents/MacOS|[t]arget/(debug|release)/tauri-app'
```

Build the current app with `npm run build`. Compare its bundle version and
executable with the installed copy before testing. A successful build does not
update `/Applications/Inbox.app`.

On a disposable draft, copy an image from a browser and press ⌘V in the capture
field. Confirm a relative Markdown image link appears, its preview loads, and
clicking the preview opens the image. Repeat with a screenshot copied directly
to the clipboard, ordinary text replacing a selection, and Edit → Paste. Test
the file picker with PNG, JPEG, and WebP files. Unsupported formats should report
an error without changing text. Test both capture and an existing entry editor.

Save a test entry, quit and reopen the same app bundle, and confirm its link and
preview survive. Verify the asset lives under `~/.inboxapp/assets/<entry-id>/`.
Deleting the test entry should move both Markdown and assets into its own trash
bundle. Discard unsaved test drafts when finished. Preserve user drafts and
clipboard contents; do not overwrite them to prepare a test.

Native app automation is available in Codex through `cua_repl`: select Inbox by
its full bundle path, inspect its current accessibility tree, focus the editor,
send `super+v`, and inspect the updated tree. Click the preview and inspect the
image viewer. Refresh the tree after changes or user interaction and derive
element indices from that observation. This drives the actual WKWebView and
native Tauri commands. It is an agent-driven smoke test, not an unattended CI
test; no native end-to-end CI harness is currently configured.

## Installing a tested local build

Quit Inbox before replacing it. Back up the existing application and
`~/.inboxapp` first, especially across a storage migration. Then install the
tested bundle:

```sh
ditto src-tauri/target/release/bundle/macos/Inbox.app /Applications/Inbox.app
shasum src-tauri/target/release/bundle/macos/Inbox.app/Contents/MacOS/tauri-app /Applications/Inbox.app/Contents/MacOS/tauri-app
```

The hashes must match. Open `/Applications/Inbox.app`, verify its version, and
repeat the native smoke test. Do not run an old app against upgraded storage.

## 26 October 2026 diagnosis

The reported “Paste failed” was reproduced in the installed **0.2.3**. That
version intercepted ⌘V and requested clipboard text only. The same clipboard
image pasted and opened successfully in the built **0.2.4**, which introduced
image support. Installing the tested 0.2.4 resolved this report without a code
change. The native tests exposed an installed-version mismatch that mocked
frontend tests could not catch. Arbitrary file capture is tracked separately in
Beads as “Capture arbitrary file attachments including PDFs” (`inboxapp-d1l`).
