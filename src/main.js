import {
  buildRecentDates,
  formatViewDate,
  visibleDateCount,
} from "./day-strip.js";

const { invoke } = window.__TAURI__.core;

const DAY_TILE_SIZE = 18;
const DAY_TILE_GAP = 0;

const state = {
  todayFilename: "",
  todayDate: "",
  todayItems: [],
  viewDate: "",
  viewItems: [],
  dayCounts: new Map(),
  drawerOpen: false,
  error: null,
  saving: false,
  statusTimer: null,
};

const editorEl = document.querySelector("#editor");
const drawerEl = document.querySelector(".drawer");
const toggleEl = document.querySelector("#drawer-toggle");
const listEl = document.querySelector("#today-list");
const countEl = document.querySelector("#today-count");
const copyAllEl = document.querySelector("#copy-all");
const statusEl = document.querySelector("#drawer-status");
const drawerTitleEl = document.querySelector(".drawer__title");
const backToTodayEl = document.querySelector("#back-to-today");
const dayStripEl = document.querySelector("#day-strip");

function setDrawerState(open) {
  state.drawerOpen = open;
  drawerEl.dataset.state = open ? "expanded" : "collapsed";
}

function clearEditor() {
  editorEl.value = "";
}

function focusEditor() {
  editorEl.focus();
}

function setError(message) {
  state.error = message;
  if (message) {
    statusEl.textContent = message;
  } else if (!state.statusTimer) {
    statusEl.textContent = "";
  }
}

function showStatus(message) {
  if (state.error) {
    return;
  }
  if (state.statusTimer) {
    clearTimeout(state.statusTimer);
  }
  statusEl.textContent = message;
  state.statusTimer = setTimeout(() => {
    statusEl.textContent = "";
    state.statusTimer = null;
  }, 1400);
}

function updateCount() {
  const total = state.viewItems.length;
  countEl.textContent = String(total);
  copyAllEl.disabled = total === 0;
}

function updateHeader() {
  if (!state.viewDate) {
    return;
  }
  const isToday = state.viewDate === state.todayDate;
  drawerTitleEl.textContent = isToday
    ? "Today"
    : formatViewDate(state.viewDate);
  backToTodayEl.hidden = isToday;
}

function displayText(item) {
  return item.replace(/\s+/g, " ").trim();
}

function todayString() {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function dateFromFilename(filename) {
  return filename.replace(/\.md$/, "");
}

function renderList() {
  listEl.textContent = "";
  const items = state.viewItems.slice().reverse();
  items.forEach((item) => {
    const row = document.createElement("li");
    row.className = "drawer__item";

    const text = document.createElement("span");
    text.className = "drawer__item-text";
    text.textContent = displayText(item);

    const copy = document.createElement("button");
    copy.type = "button";
    copy.className = "drawer__item-copy";
    copy.textContent = "Copy";
    copy.addEventListener("click", (event) => {
      event.stopPropagation();
      copyText(item);
    });

    row.append(text, copy);
    listEl.append(row);
  });
}

function formatDayCount(count) {
  if (count > 99) {
    return "99";
  }
  return String(count);
}

function dayCountLevel(count) {
  if (count === 0) {
    return 0;
  }
  if (count <= 2) {
    return 1;
  }
  if (count <= 5) {
    return 2;
  }
  if (count <= 9) {
    return 3;
  }
  return 4;
}

function renderDayStrip() {
  if (!dayStripEl || !state.todayDate) {
    return;
  }

  const visibleCount = visibleDateCount(
    dayStripEl.clientWidth,
    DAY_TILE_SIZE,
    DAY_TILE_GAP
  );

  if (!visibleCount) {
    return;
  }

  let oldestNonzeroDate = null;
  for (const [date, count] of state.dayCounts.entries()) {
    if (count > 0 && (!oldestNonzeroDate || date < oldestNonzeroDate)) {
      oldestNonzeroDate = date;
    }
  }
  if (!oldestNonzeroDate) {
    oldestNonzeroDate = state.todayDate;
  }

  const recentDates = buildRecentDates(state.todayDate, visibleCount);
  dayStripEl.textContent = "";

  for (const date of recentDates) {
    if (date < oldestNonzeroDate) {
      break;
    }
    const count = state.dayCounts.get(date) ?? 0;
    const level = dayCountLevel(count);
    const button = document.createElement("button");

    button.type = "button";
    button.className = `day-strip__day day-strip__day--level-${level}`;
    if (date === state.viewDate) {
      button.classList.add("day-strip__day--active");
    }
    button.textContent = count === 0 ? "" : formatDayCount(count);
    button.dataset.date = date;
    button.setAttribute(
      "aria-label",
      `${formatViewDate(date)}: ${count} item${count === 1 ? "" : "s"}`
    );
    button.addEventListener("click", () => {
      setViewDate(date);
    });

    dayStripEl.append(button);
  }
}

function syncTodayCount() {
  if (!state.todayDate) {
    return;
  }
  state.dayCounts.set(state.todayDate, state.todayItems.length);
}

async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text);
    showStatus("Copied");
  } catch (error) {
    console.error("clipboard copy failed", error);
    setError("Copy failed");
  }
}

async function pasteFromClipboard() {
  try {
    const text = await invoke("plugin:clipboard-manager|read_text");
    if (!text) {
      return;
    }
    const start = editorEl.selectionStart;
    const end = editorEl.selectionEnd;
    const value = editorEl.value;
    editorEl.value = value.slice(0, start) + text + value.slice(end);
    const cursor = start + text.length;
    editorEl.setSelectionRange(cursor, cursor);
  } catch (error) {
    console.error("clipboard read failed", error);
    setError("Paste failed");
  }
}

async function loadTodayItems() {
  const data = await invoke("get_today_items");
  const previousToday = state.todayDate;

  state.todayFilename = data.filename;
  state.todayDate = dateFromFilename(data.filename);
  state.todayItems = data.items;
  syncTodayCount();

  if (!state.viewDate || state.viewDate === previousToday) {
    state.viewDate = state.todayDate;
    state.viewItems = state.todayItems;
  }

  if (state.viewDate === state.todayDate) {
    renderList();
    updateCount();
  }

  updateHeader();
  renderDayStrip();
  setError(null);
}

async function loadDayCounts() {
  try {
    const counts = await invoke("list_day_counts");
    state.dayCounts = new Map(counts.map((entry) => [entry.date, entry.count]));
    syncTodayCount();
    renderDayStrip();
  } catch (error) {
    console.error("list_day_counts failed", error);
  }
}

async function refreshTodayIfStale() {
  const today = todayString();
  if (state.todayDate === today) {
    return false;
  }
  try {
    await loadTodayItems();
    return true;
  } catch (error) {
    console.error("get_today_items failed", error);
    setError("Load failed");
    return false;
  }
}

async function setViewDate(date) {
  if (state.viewDate === date) {
    return;
  }

  if (date === state.todayDate) {
    state.viewDate = date;
    state.viewItems = state.todayItems;
    renderList();
    updateCount();
    updateHeader();
    renderDayStrip();
    return;
  }

  try {
    const data = await invoke("get_items_for_date", { date });
    state.viewDate = data.date;
    state.viewItems = data.items;
    renderList();
    updateCount();
    updateHeader();
    renderDayStrip();
    setError(null);
  } catch (error) {
    console.error("get_items_for_date failed", error);
    setError("Load failed");
  }
}

async function handleBlur() {
  const rawText = editorEl.value;
  if (!rawText.trim()) {
    clearEditor();
    return;
  }
  if (state.saving) {
    return;
  }
  await refreshTodayIfStale();
  state.saving = true;
  const trimmedText = rawText.replace(/\s+$/, "");

  try {
    await invoke("append_today_item", {
      text: trimmedText,
    });
    state.todayItems.push(trimmedText);
    syncTodayCount();

    if (state.viewDate === state.todayDate) {
      state.viewItems = state.todayItems;
      updateCount();
      renderList();
    } else {
      await setViewDate(state.viewDate);
    }

    renderDayStrip();
    clearEditor();
    setError(null);
  } catch (error) {
    console.error("append_today_item failed", error);
    setError("Save failed");
  } finally {
    state.saving = false;
  }
}

async function handleFocus() {
  await refreshTodayIfStale();
  clearEditor();
  focusEditor();
}

async function boot() {
  try {
    await loadTodayItems();
    await loadDayCounts();
    setDrawerState(false);
    focusEditor();
  } catch (error) {
    console.error("get_today_items failed", error);
    setError("Load failed");
  }
}

toggleEl.addEventListener("click", () => {
  setDrawerState(!state.drawerOpen);
});

backToTodayEl.addEventListener("click", () => {
  setViewDate(state.todayDate);
});

copyAllEl.addEventListener("click", () => {
  if (!state.viewItems.length) {
    return;
  }
  const joined = state.viewItems.join("\n\n---\n\n");
  copyText(joined);
});

window.addEventListener("blur", handleBlur);
window.addEventListener("focus", handleFocus);
window.addEventListener("resize", renderDayStrip);
window.addEventListener("DOMContentLoaded", boot);
editorEl.addEventListener("keydown", (event) => {
  if (event.metaKey && event.key.toLowerCase() === "v") {
    event.preventDefault();
    event.stopPropagation();
    pasteFromClipboard();
  }
});
