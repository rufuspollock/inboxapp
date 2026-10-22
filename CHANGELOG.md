## 0.2.3 (2026-10-22)

- Save captures explicitly with Save or ⌘Enter; drafts survive app switching and restarts.
- Edit entries from any day with Save/Cancel and a date picker for older journals.
- Keep drafts and edits intact when a disk write fails.

## [0.2.4](https://github.com/rufuspollock/inboxapp/compare/v0.2.3...v0.2.4) (2026-10-22)


### Features

* add archive/restore commands with verification ([22378c9](https://github.com/rufuspollock/inboxapp/commit/22378c9ec88eb906c212995eeb83c836d9c11739))
* add archive/restore helpers with verification ([c264135](https://github.com/rufuspollock/inboxapp/commit/c264135c4a355315c76327a6ece6327bfa7042d5))
* add capture and today drawer layout ([6e830d1](https://github.com/rufuspollock/inboxapp/commit/6e830d14f7a307cdb000f4699a8e0cfabbaa75da))
* add checkbox and delete styles ([f67a0f4](https://github.com/rufuspollock/inboxapp/commit/f67a0f436cc1fb38f18ae2295751065c75e16860))
* add checkbox completion and delete actions ([56d4607](https://github.com/rufuspollock/inboxapp/commit/56d4607e8d7151351900e16646ea9c55b4e37cbb))
* add day counts listing for strip ([5e7c743](https://github.com/rufuspollock/inboxapp/commit/5e7c74343ea5f4cda8f60a3fc7e59f5e65b11f34))
* add day strip and view-date switching ([f437bd4](https://github.com/rufuspollock/inboxapp/commit/f437bd42c75f7d5ccf9f404d0c9b24e7337f155c))
* add day strip utilities ([1471b25](https://github.com/rufuspollock/inboxapp/commit/1471b255735f7f720e77c5c7b5cf07556585e106))
* add landing page ([ea41ff7](https://github.com/rufuspollock/inboxapp/commit/ea41ff77f6c35bcab03c3461359459521eb6b908))
* add markdown item parsing and append API ([31aaa2e](https://github.com/rufuspollock/inboxapp/commit/31aaa2ee33d150708345eb2993011b2842f32304))
* add menubar toggle ([5990293](https://github.com/rufuspollock/inboxapp/commit/59902930ce933d232e24e52072e7490f354b042c))
* add renderer layer and archive toggle ([12877fc](https://github.com/rufuspollock/inboxapp/commit/12877fc8a591c73c8cc1ca24cfe28f29728d34b8))
* add storage helpers ([fa95635](https://github.com/rufuspollock/inboxapp/commit/fa9563548eaa3a38b3b884e800929c1f27e1e658))
* add tauri commands ([1c26a6b](https://github.com/rufuspollock/inboxapp/commit/1c26a6b6ed50363efc3d6944d15bda7f30a41659))
* add umami analytics. ([3d900b7](https://github.com/rufuspollock/inboxapp/commit/3d900b7a0e8cec1222552f8bf8baa54046351b23))
* add view-date and day-count commands ([686210a](https://github.com/rufuspollock/inboxapp/commit/686210a22e4249eade459fec4b7022e9691ba02c))
* apply minimal editor styling ([18b7b71](https://github.com/rufuspollock/inboxapp/commit/18b7b71b67d7a1ed2672af3e7c3f89aff026e75c))
* automate releases with release-please ([8ca034a](https://github.com/rufuspollock/inboxapp/commit/8ca034a03ea87b80d1fa0138ab2dfdd8fa9c802b))
* build minimal ui ([2e3f6a2](https://github.com/rufuspollock/inboxapp/commit/2e3f6a2f6fba713f4e4c5b29dc828e42629ad350))
* copy checklist markdown formatting ([fd1343c](https://github.com/rufuspollock/inboxapp/commit/fd1343c89269db7ae8d4bd44c39bfa58f26f44dd))
* hide menubar count ([37a20fc](https://github.com/rufuspollock/inboxapp/commit/37a20fcf204ea661dfe0cfba1316d27fb6161a52))
* implement blur commit and drawer behavior ([0a8c9e4](https://github.com/rufuspollock/inboxapp/commit/0a8c9e42d834137796ec4751a2a4d245b906ca33))
* read items for date without creating file ([8abf84c](https://github.com/rufuspollock/inboxapp/commit/8abf84c8c15b978b4eac09be0453d897904be8a9))
* remove list and counts from ui ([576a3c1](https://github.com/rufuspollock/inboxapp/commit/576a3c1dd769ca9d76ce9d1711eecd9f0a7c380d))
* render formatted text with archive interactions ([03a0b24](https://github.com/rufuspollock/inboxapp/commit/03a0b249505701d149a76ad4c99199c5c6e9af3e))
* save captures explicitly and edit entries from any day ([c79fb68](https://github.com/rufuspollock/inboxapp/commit/c79fb684a27afd5b5c7b969dde205dae7658af8e))
* show count in menubar title ([62c137a](https://github.com/rufuspollock/inboxapp/commit/62c137aca0a5868c8df4cf7d3a8a17cb4775b040))
* simplify editor layout ([1e4b79f](https://github.com/rufuspollock/inboxapp/commit/1e4b79f8adcd0b2aac8f6995d52b2c3a6734f164))
* update and delete items in storage ([33a4b35](https://github.com/rufuspollock/inboxapp/commit/33a4b351640176d05bcf08825e566514b16c45eb))
* update tray icon and count title ([63d8e02](https://github.com/rufuspollock/inboxapp/commit/63d8e02057e03c5b90c96cb512eae82c6a5a0753))
* update window title with today count ([5722f45](https://github.com/rufuspollock/inboxapp/commit/5722f45214fa4dd01cebb76425ca83212a243460))


### Bug Fixes

* avoid restore controls on blank archived lines ([85c0f14](https://github.com/rufuspollock/inboxapp/commit/85c0f14f2f12f3056a6a6a69c3bcb9dc580c0ec0))
* configure root application for manifest releases ([c648688](https://github.com/rufuspollock/inboxapp/commit/c64868877e5b7b34a7673610193098ccd76bea4a))
* layer renderer for checkbox clicks ([eb43417](https://github.com/rufuspollock/inboxapp/commit/eb4341721da5816e6f97838b7c894c756aaf7ea4))
* make Cmd+V paste work on macOS ([1a8bb80](https://github.com/rufuspollock/inboxapp/commit/1a8bb8051a52a77005c98e1e4035a488630db923))

### 2026-01-13 ✨

- Copying items now outputs Markdown checkboxes (checked state preserved).
- Copy all adds a `### YYYY-MM-DD` heading and formats items as a checklist.
- Multi-paragraph items copy with proper list indentation and blank-line spacing.

### 2026-01-07 ✨

- ✨ Add checkbox-based task completion with hover-only delete controls; checked items persist as `- [x]` in daily Markdown files, and deletions are safely moved into a rolling `trash.md` for recovery.
- Add a bottom swim-lane day strip that surfaces recent days as tiny squares with count-based color intensity.
- Clicking a day square switches the Today drawer to that day while keeping the capture editor writing to today.
- Show “Today” label for the active day, show dates for past days, and include a Back to Today affordance.
- Empty days appear as gray squares with no count, and the strip stops once there are no earlier non-empty days.

### 2026-01-06

- Fix stale daily file writes when the app stays open past midnight; appends now always target today's file.
- Show newest Today items at the top of the list.
- Lock window scrolling and hide scrollbars by scrolling within the editor and Today panel.
- Tighten the Today list rows with compact spacing and subtle separators.

### 2026-01-02 — Simplify Interface

- Blank-on-focus capture surface with blur-to-commit flow and Markdown append-only storage per day.
- Collapsible Today drawer with muted list view and copy actions for single items or all.
- Reliable Cmd+V paste on macOS via native clipboard integration.
