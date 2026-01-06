# Compact Today List Rows Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Tighten the Today list into compact single-line rows with subtle separators and no row hover border while keeping copy hover.

**Architecture:** Pure CSS adjustments in the existing drawer list styles. No markup or JS changes needed.

**Tech Stack:** HTML, CSS (vanilla).

### Task 1: Update Today list row spacing and separators

**Files:**
- Modify: `src/styles.css`

**Step 1: Write the failing test**

No automated tests for CSS changes.

**Step 2: Run test to verify it fails**

Run: `npm test`
Expected: `No tests`

**Step 3: Write minimal implementation**

Update the drawer list styles to remove item gaps, add bottom borders, drop row hover borders, and keep compact sizing.

```css
.drawer__list {
  gap: 0;
}

.drawer__item {
  border-bottom: 1px solid var(--divider);
}

.drawer__item:last-child {
  border-bottom: none;
}

.drawer__item:hover {
  border-color: transparent;
}
```

**Step 4: Run test to verify it passes**

Run: `npm test`
Expected: `No tests`

**Step 5: Commit**

```bash
git add src/styles.css
git commit -m "Tighten today list spacing and add separators"
```
