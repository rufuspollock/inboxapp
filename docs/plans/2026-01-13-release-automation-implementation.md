# Release Automation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Release Please and CI workflows to automate versioning, changelog generation, and DMG uploads, with README documentation.

**Architecture:** Use Release Please on `main` with Conventional Commits to generate release PRs, bump versions, and create tags/releases. A separate workflow triggers on `release.published` to build the macOS DMG and upload it to the GitHub Release.

**Tech Stack:** GitHub Actions, Release Please, Node.js tests (`node --test`), Tauri build.

### Task 1: Add failing test for Release Please config

**Files:**
- Create: `tests/release-please.test.js`

**Step 1: Write the failing test**

```js
import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs/promises";
import path from "node:path";

const root = path.resolve(new URL(".", import.meta.url).pathname, "..");
const configPath = path.join(root, ".github", "release-please-config.json");

function normalizePath(p) {
  return p.replace(/\\/g, "/");
}

test("release-please config includes extra files for tauri versioning", async () => {
  const raw = await fs.readFile(configPath, "utf-8");
  const config = JSON.parse(raw);
  assert.equal(config["release-type"], "node");
  assert.equal(config["package-name"], "tauri-app");
  const extraFiles = config["extra-files"].map((file) => normalizePath(file));
  assert.ok(extraFiles.includes("src-tauri/tauri.conf.json"));
  assert.ok(extraFiles.includes("src-tauri/Cargo.toml"));
});
```

**Step 2: Run test to verify it fails**

Run: `node --test tests/release-please.test.js`

Expected: FAIL with `ENOENT` for missing `.github/release-please-config.json`.

### Task 2: Add Release Please configuration

**Files:**
- Create: `.github/release-please-config.json`
- Create: `.github/release-please-manifest.json`

**Step 1: Write minimal implementation**

```json
{
  "release-type": "node",
  "package-name": "tauri-app",
  "include-component-in-tag": false,
  "include-v-in-tag": true,
  "bump-minor-pre-major": true,
  "bump-patch-for-minor-pre-major": true,
  "extra-files": [
    "src-tauri/tauri.conf.json",
    "src-tauri/Cargo.toml"
  ]
}
```

```json
{
  ".": "0.2.2"
}
```

**Step 2: Run test to verify it passes**

Run: `node --test tests/release-please.test.js`

Expected: PASS.

### Task 3: Add Release Please workflow

**Files:**
- Create: `.github/workflows/release-please.yml`

**Step 1: Write the failing test**

No automated test for workflow behavior; rely on configuration test + YAML review.

**Step 2: Write minimal implementation**

```yaml
name: Release Please
on:
  push:
    branches: [main]
permissions:
  contents: write
  pull-requests: write
jobs:
  release-please:
    runs-on: ubuntu-latest
    steps:
      - uses: google-github-actions/release-please-action@v4
        with:
          config-file: .github/release-please-config.json
          manifest-file: .github/release-please-manifest.json
```

**Step 3: Run tests to verify still pass**

Run: `npm test`

Expected: PASS.

### Task 4: Add DMG build workflow

**Files:**
- Create: `.github/workflows/build-dmg.yml`

**Step 1: Write the failing test**

No automated test; validate workflow by inspection.

**Step 2: Write minimal implementation**

```yaml
name: Build DMG
on:
  release:
    types: [published]
permissions:
  contents: write
jobs:
  build-dmg:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
      - uses: dtolnay/rust-toolchain@stable
      - run: npm install
      - run: npm run build
      - uses: softprops/action-gh-release@v2
        with:
          files: src-tauri/target/release/bundle/dmg/*.dmg
```

**Step 3: Run tests to verify still pass**

Run: `npm test`

Expected: PASS.

### Task 5: Document the release flow in README

**Files:**
- Modify: `README.md`

**Step 1: Write the failing test**

No automated test; validate by reviewing README.

**Step 2: Write minimal implementation**

Add a short "Release Flow" section with bullets:
- Use Conventional Commits on branches merged into `main`
- Release Please opens/updates a release PR
- Merge release PR to create tag + release
- DMG builds on GitHub Actions and attaches to the release

**Step 3: Run tests to verify still pass**

Run: `npm test`

Expected: PASS.

### Task 6: Commit

```bash
git add .github/workflows/release-please.yml .github/workflows/build-dmg.yml \
  .github/release-please-config.json .github/release-please-manifest.json \
  tests/release-please.test.js README.md
git commit -m "feat: automate releases with release-please"
```
