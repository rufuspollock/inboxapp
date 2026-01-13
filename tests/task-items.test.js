import assert from "node:assert/strict";
import test from "node:test";
import {
  parseTaskItem,
  formatTaskItem,
  formatMarkdownChecklistItem,
  formatMarkdownChecklist,
} from "../src/task-items.js";

test("parseTaskItem detects checked items", () => {
  const result = parseTaskItem("- [x] buy milk");
  assert.equal(result.checked, true);
  assert.equal(result.text, "buy milk");
});

test("parseTaskItem treats plain lines as unchecked", () => {
  const result = parseTaskItem("buy milk");
  assert.equal(result.checked, false);
  assert.equal(result.text, "buy milk");
});

test("formatTaskItem preserves multi-line content", () => {
  const result = formatTaskItem("line one\nline two", true);
  assert.equal(result, "- [x] line one\nline two");
});

test(
  "formatMarkdownChecklistItem renders unchecked checkbox and indents paragraphs",
  () => {
    const text = "First line\n\nSecond para\nline two";
    const result = formatMarkdownChecklistItem(text, false);
    assert.equal(result, "- [ ] First line\n\n  Second para\n  line two");
  }
);

test("formatMarkdownChecklist builds list with heading and spacing", () => {
  const items = ["Alpha", "Bravo\n\nSecond"];
  const result = formatMarkdownChecklist(items, {
    heading: "### 2026-01-13",
  });
  assert.equal(
    result,
    "### 2026-01-13\n\n- [ ] Alpha\n\n- [ ] Bravo\n\n  Second"
  );
});

test("formatMarkdownChecklist preserves checked items", () => {
  const items = ["- [x] Done", "Todo"];
  const result = formatMarkdownChecklist(items);
  assert.equal(result, "- [x] Done\n\n- [ ] Todo");
});
