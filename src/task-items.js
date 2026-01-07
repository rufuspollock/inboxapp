const TASK_MARKER = /^-\s\[( |x|X)\]\s+/;

export function parseTaskItem(item) {
  const lines = item.split("\n");
  const match = lines[0]?.match(TASK_MARKER);
  if (match) {
    lines[0] = lines[0].slice(match[0].length);
    return {
      checked: match[1].toLowerCase() === "x",
      text: lines.join("\n"),
    };
  }
  return { checked: false, text: item };
}

export function formatTaskItem(text, checked) {
  const lines = text.split("\n");
  const marker = checked ? "- [x] " : "- [ ] ";
  lines[0] = marker + (lines[0] ?? "");
  return lines.join("\n");
}
