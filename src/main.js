const { invoke } = window.__TAURI__.core;

const state = {
  filename: "",
  date: "",
  items: [],
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
  const total = state.items.length;
  countEl.textContent = String(total);
  copyAllEl.disabled = total === 0;
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
  const items = state.items.slice().reverse();
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

async function refreshTodayIfStale() {
  const today = todayString();
  if (state.date === today) {
    return false;
  }
  try {
    const data = await invoke("get_today_items");
    state.filename = data.filename;
    state.date = dateFromFilename(data.filename);
    state.items = data.items;
    renderList();
    updateCount();
    setError(null);
    return true;
  } catch (error) {
    console.error("get_today_items failed", error);
    setError("Load failed");
    return false;
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
    state.items.push(trimmedText);
    updateCount();
    renderList();
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
    const data = await invoke("get_today_items");
    state.filename = data.filename;
    state.date = dateFromFilename(data.filename);
    state.items = data.items;
    renderList();
    updateCount();
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

copyAllEl.addEventListener("click", () => {
  if (!state.items.length) {
    return;
  }
  const joined = state.items.join("\n\n---\n\n");
  copyText(joined);
});

window.addEventListener("blur", handleBlur);
window.addEventListener("focus", handleFocus);
window.addEventListener("DOMContentLoaded", boot);
editorEl.addEventListener("keydown", (event) => {
  if (event.metaKey && event.key.toLowerCase() === "v") {
    event.preventDefault();
    event.stopPropagation();
    pasteFromClipboard();
  }
});
