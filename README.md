# InboxApp

### Just capture. Nothing else.

#### Capture fast. Process later.

InboxApp is a hyper-minimal macOS inbox for capturing fleeting thoughts in Markdown with near-zero friction. It is designed to be emptied, not to become another knowledge base.

Background: [why capture and processing are separate](docs/background/capture-and-processing.md), including the earlier inbox experiment and its relationship to bookmarking.

🔗 https://tryinbox.app/

📺 Watch the demo https://www.youtube.com/watch?v=JdY6tsNhN30

## Key Features

- Single, distraction-free writing surface with raw Markdown.
- Explicit Save button or `⌘+Enter` to capture a thought into its own dated Markdown file.
- Drafts stay put when switching apps and recover after restarting.
- Click any saved entry to edit it with Save/Cancel; use the date picker to open older days.
- Paste screenshots or attach PNG, JPEG, and WebP images, with small previews.
- Local, offline storage in human-readable files at `~/.inboxapp`. Each entry owns its attachments.
- Ultra-lightweight UI that resembles a blank page (black/white, minimalist).
- Today’s item count shown in the native window title as `Inbox — N`.
- No AI, no tagging, no organization: just capture and move on.

## Install (Developer)

Use the DMG on the GitHub releases page. 

🚩 This app is not yet signed or notarized by Apple, so macOS may block it on first launch.

**Option A (recommended)**

1. In Finder, Control-click the app (or the app inside the DMG).
2. Choose **Open**.
3. When prompted, click **Open** again.

**Option B**

1. Try to open the app once (it will be blocked).
2. Go to **System Settings → Privacy & Security**.
3. Click **Open Anyway** next to the app.

**Advanced (Terminal)**

```bash
xattr -dr com.apple.quarantine /path/to/YourApp.app
```

**Notes**

* These steps create a per-app exception in Gatekeeper.
* Only proceed if you trust the source.
* Updates may require repeating the process.

### Prerequisites

- Node.js (with npm)
- Rust toolchain
- Tauri prerequisites for macOS (Xcode Command Line Tools)

### Steps
```bash
npm install
npm run dev
```

### Scripts
- `npm run dev` - Run the app in development mode.
- `npm run build` - Build a macOS installer in `src-tauri/target/release/bundle/`.

### Release Flow
- Use Conventional Commits on branches merged into `main`.
- Release Please opens or updates a release PR on `main`.
- Merge the release PR to create the `vX.Y.Z` tag and GitHub Release.
- GitHub Actions builds the DMG and attaches it to the release.

## Develop

### Run the app
```bash
npm run dev
```

### Run Rust tests
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

See [testing on macOS](docs/testing.md) for frontend tests, real clipboard and
window automation, and verifying that the installed app matches the tested build.

### Project layout
- `src/` - frontend (HTML/CSS/JS)
- `src-tauri/` - backend (Rust/Tauri)
- `~/.inboxapp/` - local storage (individual Markdown entries and owned assets)

## Design Principles
- Capture only, no processing inside the app.
- One surface, one action: write.
- Markdown-first, Obsidian-compatible output.
- Visually quiet and fast.

## Non-goals
- Long-term storage or knowledge management.
- Rich editing, rendered Markdown, or inline formatting.
- Sync, automation, or AI features.

## Storage and upgrades

Each thought is a plain Markdown file, such as `2026-10-25-0001.md`. The date and
stable per-day sequence are in its filename; the day picker groups these files.
Separators (`---`) inside new entries are ordinary Markdown, not entry boundaries.

```text
~/.inboxapp/
  config.yml                       # schema_version and next ID sequences
  2026-10-25-0001.md                # one entry
  assets/2026-10-25-0001/           # images owned by that entry
  .drafts/                         # reserved draft IDs
  .trash/2026-10-25-0001/           # deleted entry + its assets, as a bundle
  legacy-backup-v1/                # original daily files and migration manifest
```

On the first launch of this version, existing daily files upgrade automatically.
The app keeps the originals (including the old `trash.md`) in `legacy-backup-v1/`
and resumes an interrupted upgrade without duplicating entries. If files changed
during an interrupted upgrade, or the storage version is newer than this app,
it reports an error rather than overwriting anything. Keep the backup until you
have verified your migrated entries. Do not run an older app version against the
upgraded folder.

Delete moves the entry and its own image folder into `.trash/<entry-id>/`, retaining
working relative links inside that bundle. Recovery is manual for now: move the
Markdown file back to the storage root and its `assets/<entry-id>/` folder back
under root `assets/`, without overwriting existing files. Permanently removing a
trash bundle removes that entry's attachments too. Legacy trash remains in the
migration backup.

Images are written locally before their Markdown links are inserted. Screenshot
clipboard data becomes PNG; PNG/JPEG/WebP files retain their bytes. Imports are
limited to 20 MB and 40 million pixels. Draft text and its reserved entry ID recover
across restarts; **Discard draft** removes its unsaved assets. Removing an image
link or cancelling an edit keeps the entry's image files until the whole entry is
deleted, so a cancelled edit cannot break an existing link.

To move a thought elsewhere, copy its Markdown file **and** matching
`assets/<entry-id>/` folder, preserving their relative layout. Copy and Copy all
copy Markdown text only; they do not transfer image files. Pasting a copied entry’s
image links into another entry is rejected on Save; attach a fresh copy so deleting
the original cannot break it. Attachment-aware export
is not yet implemented. See [the website changelog](https://tryinbox.app/changelog/)
and [website publishing instructions](site/README.md).
