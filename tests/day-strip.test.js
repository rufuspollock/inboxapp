import assert from "node:assert/strict";
import test from "node:test";
import {
  buildRecentDates,
  formatViewDate,
  visibleDateCount,
} from "../src/day-strip.js";

test("formatViewDate formats YYYY-MM-DD", () => {
  assert.equal(formatViewDate("2026-01-07"), "Wed, Jan 7");
});

test("buildRecentDates returns descending dates", () => {
  const dates = buildRecentDates("2026-01-03", 3);
  assert.deepEqual(dates, ["2026-01-03", "2026-01-02", "2026-01-01"]);
});

test("visibleDateCount respects width", () => {
  assert.equal(visibleDateCount(200, 18, 6), 8);
});
