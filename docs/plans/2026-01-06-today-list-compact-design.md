# Today list compact rows design

- Goal: Make the Today task list read as compact, single-line rows with subtle separators while keeping the existing checklist indicator and copy button.
- Scope: CSS-only adjustments in `src/styles.css`; no behavior or data changes.
- Layout: Remove inter-item gaps (`gap: 0`) and add `border-bottom: 1px solid var(--divider)` to each `.drawer__item`; remove the last item's border for a clean end cap.
- Density: Keep compact padding, smaller font size/line-height for `.drawer__item`, and a reduced checkbox size to align with tighter rows.
- Hover: Remove the row hover border; keep copy-button hover reveal unchanged.
- Truncation: Maintain single-line truncation with existing ellipsis styles on `.drawer__item-text`.
- Accessibility: Preserve button focus and click behavior; no changes to semantics.

## Success criteria

- Tasks appear as a stacked list of rows with subtle separators and minimal vertical whitespace.
- Each item remains a single line with ellipsis when text is long.
- Copy button still appears on hover; row itself does not show a hover border.
