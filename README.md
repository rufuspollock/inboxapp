# InboxApp

### Just capture. Nothing else.

#### Capture fast. Process later.

InboxApp is a hyper-minimal macOS inbox for capturing fleeting thoughts in Markdown with near-zero friction. It is designed to be emptied, not to become another knowledge base.

🔗 https://tryinbox.app/

📺 Watch the demo https://www.youtube.com/watch?v=JdY6tsNhN30

## Key Features

- Single, distraction-free writing surface with raw Markdown.
- Explicit Save button or `⌘+Enter` to capture a thought into today’s Markdown file.
- Drafts stay put when switching apps and recover after restarting.
- Click any saved entry to edit it with Save/Cancel; use the date picker to open older days.
- Local, offline storage in human-readable files at `~/.inboxapp`.
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
cargo test
```

### Project layout
- `src/` - frontend (HTML/CSS/JS)
- `src-tauri/` - backend (Rust/Tauri)
- `~/.inboxapp/` - local storage (daily Markdown files)

## Design Principles
- Capture only, no processing inside the app.
- One surface, one action: write.
- Markdown-first, Obsidian-compatible output.
- Visually quiet and fast.

## Non-goals
- Long-term storage or knowledge management.
- Rich editing, rendered Markdown, or inline formatting.
- Sync, automation, or AI features.
