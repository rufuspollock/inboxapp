import assert from "node:assert/strict";
import test from "node:test";
import { parseTaskItem, formatTaskItem } from "../src/task-items.js";

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
