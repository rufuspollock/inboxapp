# InboxApp

Capture without thinking. Process later.

InboxApp is a hyper-minimal macOS inbox for capturing fleeting thoughts in Markdown with near-zero friction. It is designed to be emptied, not to become another knowledge base.

## Key Features
- Single, distraction-free writing surface with raw Markdown.
- Today-only capture buffer (daily file) with the cursor placed at the end on open.
- Local, offline storage in human-readable files at `~/.inboxapp`.
- Ultra-lightweight UI that resembles a blank page (black/white, minimalist).
- Today’s item count shown in the native window title as `Inbox — N`.
- No AI, no tagging, no organization: just capture and move on.

## Install (Developer)

For now, install is developer-only. A packaged installer will come later.

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
