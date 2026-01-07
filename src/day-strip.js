const LABEL_FORMATTER = new Intl.DateTimeFormat("en-US", {
  weekday: "short",
  month: "short",
  day: "numeric",
});

export function formatViewDate(dateString) {
  const [year, month, day] = dateString.split("-").map(Number);
  const date = new Date(year, month - 1, day);
  return LABEL_FORMATTER.format(date);
}

export function buildRecentDates(todayString, count) {
  const [year, month, day] = todayString.split("-").map(Number);
  const date = new Date(year, month - 1, day);
  const out = [];

  for (let i = 0; i < count; i += 1) {
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, "0");
    const d = String(date.getDate()).padStart(2, "0");
    out.push(`${y}-${m}-${d}`);
    date.setDate(date.getDate() - 1);
  }

  return out;
}

export function visibleDateCount(containerWidth, size, gap) {
  if (!containerWidth) {
    return 0;
  }
  return Math.max(1, Math.floor((containerWidth + gap) / (size + gap)));
}
