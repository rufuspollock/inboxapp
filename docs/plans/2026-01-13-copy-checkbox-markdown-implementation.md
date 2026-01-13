# Copy Checkbox Markdown Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Copy actions output Markdown checkboxes, with copy-all including a dated heading and correct multi-paragraph indentation.

**Architecture:** Add pure formatting helpers in `src/task-items.js` for Markdown checklist output, reuse them in `src/main.js` copy handlers, and cover them with unit tests. Keep UI and clipboard wiring unchanged, only swap the formatted string.

**Tech Stack:** Node.js tests (`node --test`), vanilla JS frontend, Tauri clipboard.

### Task 1: Add failing tests for markdown copy formatting

**Files:**
- Modify: `tests/task-items.test.js`

**Step 1: Write the failing test**

```js
import assert from "node:assert/strict";
import test from "node:test";
import {
  parseTaskItem,
  formatTaskItem,
  formatMarkdownChecklistItem,
  formatMarkdownChecklist,
} from "../src/task-items.js";

// ...existing tests...

test("formatMarkdownChecklistItem renders unchecked checkbox and indents paragraphs", () => {
  const text = "First line\n\nSecond para\nline two";
  const result = formatMarkdownChecklistItem(text, false);
  assert.equal(
    result,
    "- [ ] First line\n\n  Second para\n  line two"
  );
});

test("formatMarkdownChecklist builds list with heading and spacing", () => {
  const items = ["Alpha", "Bravo\n\nSecond"];
  const result = formatMarkdownChecklist(items, {
    heading: "### 2026-01-13",
    checked: false,
  });
  assert.equal(
    result,
    "### 2026-01-13\n\n- [ ] Alpha\n\n- [ ] Bravo\n\n  Second"
  );
});
```

**Step 2: Run test to verify it fails**

Run: `node --test tests/task-items.test.js`

Expected: FAIL with `formatMarkdownChecklistItem is not a function` (or similar missing export).

### Task 2: Implement markdown checklist formatting helpers

**Files:**
- Modify: `src/task-items.js`

**Step 1: Write minimal implementation**

```js
export function formatMarkdownChecklistItem(text, checked) {
  const lines = text.split("\n");
  const marker = checked ? "- [x] " : "- [ ] ";
  const out = [];
  let inParagraph = false;

  lines.forEach((line, index) => {
    if (index === 0) {
      out.push(marker + (line ?? ""));
      inParagraph = true;
      return;
    }
    if (line.trim() === "") {
      out.push("");
      inParagraph = false;
      return;
    }
    if (!inParagraph) {
      out.push("  " + line);
      inParagraph = true;
      return;
    }
    out.push("  " + line);
  });

  return out.join("\n");
}

export function formatMarkdownChecklist(items, { heading, checked }) {
  const parts = [];
  if (heading) {
    parts.push(heading, "");
  }
  items.forEach((item, index) => {
    if (index > 0) {
      parts.push("");
    }
    parts.push(formatMarkdownChecklistItem(item, checked));
  });
  return parts.join("\n");
}
```

**Step 2: Run test to verify it passes**

Run: `node --test tests/task-items.test.js`

Expected: PASS.

### Task 3: Wire copy actions to markdown formatting

**Files:**
- Modify: `src/main.js`

**Step 1: Write the failing test**

No new unit test for UI glue. Rely on formatter unit tests.

**Step 2: Write minimal implementation**

```js
import {
  formatTaskItem,
  parseTaskItem,
  formatMarkdownChecklistItem,
  formatMarkdownChecklist,
} from "./task-items.js";

function todayHeading() {
  return `### ${todayString()}`;
}

// in per-item copy handler
const parsed = parseTaskItem(item);
copyText(formatMarkdownChecklistItem(parsed.text, parsed.checked));

// in copy-all handler
const joined = formatMarkdownChecklist(state.viewItems, {
  heading: todayHeading(),
  checked: false,
});
copyText(joined);
```

**Step 3: Run tests to verify still pass**

Run: `npm test`

Expected: PASS.

### Task 4: Refactor and tidy

**Files:**
- Modify: `src/task-items.js`

**Step 1: Light refactor**

- Ensure helper functions are exported in a stable order.
- Keep `formatTaskItem` behavior unchanged.

**Step 2: Run tests**

Run: `npm test`

Expected: PASS.

### Task 5: Commit

```bash
git add tests/task-items.test.js src/task-items.js src/main.js
git commit -m "feat: copy checklist markdown formatting"
```
