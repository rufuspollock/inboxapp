import { installAttachmentInput, renderPreviews, attachmentPicker } from "./attachments.js";
import {
  buildRecentDates,
  formatViewDate,
  visibleDateCount,
} from "./day-strip.js";
import {
  formatTaskItem,
  parseTaskItem,
  formatMarkdownChecklistItem,
  formatMarkdownChecklist,
} from "./task-items.js";

const { invoke } = window.__TAURI__.core;

const DAY_TILE_SIZE = 18;
const DAY_TILE_GAP = 0;

const state = {
  ready: false,
  importing: false,
  draftId: null,
  todayFilename: "",
  todayDate: "",
  todayItems: [],
  viewDate: "",
  viewItems: [],
  dayCounts: new Map(),
  drawerOpen: false,
  error: null,
  saving: false,
  editing: null,
  itemSaving: false,
  statusTimer: null,
};

const editorEl = document.querySelector("#editor");
const saveCaptureEl = document.querySelector("#save-capture");
const captureStatusEl = document.querySelector("#capture-status");
const DRAFT_KEY = "inboxapp.capture-draft";
const DRAFT_ID_KEY = "inboxapp.capture-draft-id";
const capturePreviewsEl = document.querySelector('#capture-previews');
const attachCaptureEl = document.querySelector('#attach-capture');
const discardCaptureEl = document.querySelector('#discard-capture');
const drawerEl = document.querySelector(".drawer");
const toggleEl = document.querySelector("#drawer-toggle");
const listEl = document.querySelector("#today-list");
const countEl = document.querySelector("#today-count");
const copyAllEl = document.querySelector("#copy-all");
const statusEl = document.querySelector("#drawer-status");
const drawerTitleEl = document.querySelector(".drawer__title");
const backToTodayEl = document.querySelector("#back-to-today");
const historyDateEl = document.querySelector("#history-date");
let viewRequest = 0;
const dayStripEl = document.querySelector("#day-strip");

function setDrawerState(open) {
  state.drawerOpen = open;
  drawerEl.dataset.state = open ? "expanded" : "collapsed";
}

function updateCaptureControls() {
  saveCaptureEl.disabled = !state.ready || state.saving || state.itemSaving || state.importing || !editorEl.value.trim();
  editorEl.readOnly = state.saving || state.importing;
  attachCaptureEl.disabled = !state.ready || state.saving || state.itemSaving || state.importing;
  discardCaptureEl.disabled = attachCaptureEl.disabled;
  discardCaptureEl.hidden = !editorEl.value && !state.draftId;
  saveCaptureEl.textContent = state.saving ? "Saving…" : "Save";
}

function persistDraft() {
  try {
    if (editorEl.value) window.localStorage.setItem(DRAFT_KEY, editorEl.value);
    else window.localStorage.removeItem(DRAFT_KEY);
  } catch (error) {
    captureStatusEl.textContent = "Draft recovery unavailable. Save before quitting.";
  }
  updateCaptureControls();
  renderPreviews(capturePreviewsEl, editorEl.value, state.draftId, invoke);
}

function clearEditor() {
  editorEl.value = "";
  state.draftId = null;
  try { window.localStorage.removeItem(DRAFT_ID_KEY); } catch {}
  persistDraft();
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
  historyDateEl.value = state.viewDate;
  historyDateEl.max = state.todayDate;
  const isToday = state.viewDate === state.todayDate;
  drawerTitleEl.textContent = isToday
    ? "Today"
    : formatViewDate(state.viewDate);
  backToTodayEl.hidden = isToday;
}

function displayText(item) {
  return item.replace(/\s+/g, " ").trim();
}

function updateItemAt(id, text) {
  state.viewItems = state.viewItems.map(entry => entry.id === id ? {...entry, text} : entry);
  if (state.viewDate === state.todayDate) state.todayItems = state.viewItems;
}

function setItemsForView(items) {
  state.viewItems = items;
  if (state.viewDate === state.todayDate) {
    state.todayItems = items;
  }
  state.dayCounts.set(state.viewDate, items.length);
}

function todayString() {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function todayHeading() {
  return `### ${todayString()}`;
}

function dateFromFilename(filename) {
  return filename.replace(/\.md$/, "");
}

function renderList() {
  listEl.textContent = "";
  const items = state.viewItems.slice().reverse();
  items.forEach((entry) => {
    const { id, text: item } = entry;
    const parsed = parseTaskItem(item);
    const row = document.createElement("li");
    row.className = "drawer__item";
    if (parsed.checked) {
      row.classList.add("drawer__item--checked");
    }

    if (state.editing?.date === state.viewDate && state.editing.id === id) {
      renderEditRow(row);
      listEl.append(row);
      return;
    }

    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.className = "drawer__item-checkbox";
    checkbox.checked = parsed.checked;
    checkbox.disabled = !!state.editing || state.itemSaving || state.importing;
    checkbox.setAttribute("aria-label", "Mark task as done");
    checkbox.addEventListener("change", async (event) => {
      event.stopPropagation();
      if (state.saving || state.itemSaving || state.importing || state.editing) return;
      state.itemSaving = true;
      ++viewRequest;
      updateCaptureControls();
      checkbox.disabled = true;
      renderList();
      const updatedItem = formatTaskItem(parsed.text, checkbox.checked);
      try {
        await invoke("update_item_for_date", {
          date: state.viewDate,
          id,
          item: updatedItem,
          expectedItem: item,
        });
        updateItemAt(id, updatedItem);
        renderList();
        setError(null);
      } catch (error) {
        console.error("update_item_for_date failed", error);
        checkbox.checked = parsed.checked;
        setError("Save failed");
      } finally {
        state.itemSaving = false;
        updateCaptureControls();
        renderList();
      }
    });

    const text = document.createElement("button");
    text.type = "button";
    text.className = "drawer__item-text";
    text.textContent = displayText(parsed.text);
    text.title = "Edit entry";
    text.disabled = state.saving || state.itemSaving || state.importing;
    text.setAttribute("aria-label", `Edit entry: ${displayText(parsed.text)}`);
    text.addEventListener("click", () => {
      if (state.saving || state.itemSaving || state.importing) return;
      if (state.editing) {
        setError("Save or cancel the current edit first");
        return;
      }
      ++viewRequest;
      state.editing = { date: state.viewDate, id, original: item, text: parsed.text, checked: parsed.checked };
      renderList();
      listEl.querySelector(".drawer__item-editor")?.focus();
    });

    const copy = document.createElement("button");
    copy.type = "button";
    copy.className = "drawer__item-copy";
    copy.textContent = "Copy";
    copy.addEventListener("click", (event) => {
      event.stopPropagation();
      const parsed = parseTaskItem(item);
      copyText(formatMarkdownChecklistItem(parsed.text, parsed.checked));
    });

    const del = document.createElement("button");
    del.type = "button";
    del.className = "drawer__item-delete";
    del.textContent = "Delete";
    del.disabled = !!state.editing || state.itemSaving || state.importing;
    del.addEventListener("click", async (event) => {
      event.stopPropagation();
      if (state.saving || state.itemSaving || state.importing || state.editing) return;
      state.itemSaving = true;
      ++viewRequest;
      updateCaptureControls();
      renderList();
      try {
        const updated = await invoke("delete_item_for_date", {
          date: state.viewDate,
          id,
          expectedItem: item,
        });
        setItemsForView(updated.items);
        updateCount();
        renderList();
        renderDayStrip();
        setError(null);
      } catch (error) {
        console.error("delete_item_for_date failed", error);
        setError("Delete failed");
      } finally {
        state.itemSaving = false;
        updateCaptureControls();
        renderList();
      }
    });

    const content = document.createElement('div');
    content.className = 'drawer__item-content';
    const previews = document.createElement('div');
    previews.className = 'attachment-previews';
    renderPreviews(previews, item, id, invoke);
    content.append(text, previews);
    row.append(checkbox, content, copy, del);
    listEl.append(row);
  });
}

function renderEditRow(row) {
  row.classList.add("drawer__item--editing");
  row.classList.remove("drawer__item--checked");
  const edit = state.editing;
  const field = document.createElement("textarea");
  field.className = "drawer__item-editor";
  field.value = edit.text;
  field.rows = Math.min(8, Math.max(3, edit.text.split("\n").length));
  field.setAttribute("aria-label", "Edit entry");
  field.readOnly = state.itemSaving || state.importing;
  const actions = document.createElement("div");
  actions.className = "drawer__item-actions";
  const save = document.createElement("button");
  save.className = "drawer__item-save";
  save.type = "button";
  save.textContent = state.itemSaving ? "Saving…" : "Save";
  save.disabled = state.itemSaving || state.importing || !edit.text.trim();
  const cancel = document.createElement("button");
  cancel.className = "drawer__item-cancel";
  cancel.type = "button";
  cancel.textContent = "Cancel";
  cancel.disabled = state.itemSaving || state.importing;
  field.addEventListener("input", () => {
    edit.text = field.value;
    save.disabled = !edit.text.trim() || state.itemSaving || state.importing;
    renderPreviews(previews, edit.text, edit.id, invoke);
  });
  cancel.addEventListener("click", async () => {
    if (state.importing) return;
    state.editing = null;
    setError(null);
    renderList();
    const date = state.viewDate;
    const request = ++viewRequest;
    try {
      const data = await invoke("get_items_for_date", { date });
      if (request !== viewRequest || state.viewDate !== date || state.editing || state.itemSaving || state.saving) return;
      setItemsForView(data.items);
      renderList();
      updateCount();
      renderDayStrip();
    } catch (error) {
      setError("Refresh failed");
    }
  });
  save.addEventListener("click", saveEditedItem);
  field.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      saveEditedItem();
    } else if (event.key === "Escape" && !state.itemSaving && !state.importing) cancel.click();
  });
  const previews = document.createElement('div');
  previews.className = 'attachment-previews';
  renderPreviews(previews, edit.text, edit.id, invoke);
  const imageInput = installAttachmentInput(field, {
    invoke, getId: async () => edit.id, isBusy: imageBusy, setBusy: setImageBusy,
    onChange: () => { edit.text = field.value; renderPreviews(previews, edit.text, edit.id, invoke); },
    onStatus: message => setError(message || null),
  });
  const picker = attachmentPicker(document, imageInput.importFiles);
  const attach = document.createElement('button');
  attach.type = 'button';
  attach.textContent = 'Attach file';
  attach.disabled = state.itemSaving || state.importing;
  attach.addEventListener('click', () => picker.click());
  actions.append(attach, save, cancel);
  row.append(field, previews, picker, actions);
}

async function saveEditedItem() {
  const edit = state.editing;
  if (!edit || state.saving || state.itemSaving || state.importing || !edit.text.trim()) return;
  state.itemSaving = true;
      ++viewRequest;
  updateCaptureControls();
  renderList();
  try {
    const item = formatTaskItem(edit.text.replace(/\s+$/, ""), edit.checked);
    const updated = await invoke("update_item_for_date", { date: edit.date, id: edit.id, item, expectedItem: edit.original });
    if (state.viewDate === edit.date) setItemsForView(updated.items);
    if (state.todayDate === edit.date) state.todayItems = updated.items;
    state.editing = null;
    setError(null);
    showStatus("Saved");
  } catch (error) {
    console.error("update_item_for_date failed", error);
    setError(String(error).includes("changed elsewhere") || String(error).includes("no longer exists")
      ? "Entry changed elsewhere. Cancel, reopen the day, then edit again."
      : `Save failed — your edit is still here. ${String(error.message || error)}`);
  } finally {
    state.itemSaving = false;
    updateCaptureControls();
    renderList();
  }
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

function imageBusy() { return !state.ready || state.saving || state.itemSaving || state.importing; }

function setImageBusy(busy) {
  state.importing = busy;
  updateCaptureControls();
  if (busy) {
    listEl.querySelectorAll('button:not(.attachment-preview), input').forEach(element => {element.disabled = true;});
    listEl.querySelectorAll('textarea').forEach(element => {element.readOnly = true;});
  } else if (state.editing) {
    const field = listEl.querySelector('.drawer__item-editor');
    if (field) field.readOnly = false;
    listEl.querySelectorAll('.drawer__item-actions button').forEach(button => {button.disabled = false;});
    const save = listEl.querySelector('.drawer__item-save');
    if (save) save.disabled = !state.editing.text.trim();
  } else renderList();
}

async function ensureDraftId() {
  if (state.draftId) return state.draftId;
  const id = await invoke('new_entry_id');
  // Persist ownership before importing bytes; a recovered draft must find its images.
  try { window.localStorage.setItem(DRAFT_ID_KEY, id); }
  catch { await invoke('discard_draft', {id}); throw new Error('Draft recovery unavailable. Restart before attaching files.'); }
  state.draftId = id;
  return id;
}

async function discardCapture() {
  if (imageBusy()) return;
  state.importing = true;
  updateCaptureControls();
  try {
    if (state.draftId) await invoke('discard_draft', {id: state.draftId});
    clearEditor();
    captureStatusEl.textContent = 'Draft discarded';
  } catch (error) { captureStatusEl.textContent = `Could not discard draft — ${String(error.message || error)}`; }
  finally { state.importing = false; updateCaptureControls(); }
}

async function loadTodayItems() {
  const data = await invoke("get_today_items");
  const previousToday = state.todayDate;

  state.todayFilename = data.filename;
  state.todayDate = dateFromFilename(data.filename);
  state.todayItems = data.items;
  syncTodayCount();

  if (!state.viewDate || (state.viewDate === previousToday && !state.editing)) {
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
  const request = ++viewRequest;
  if (state.editing && date !== state.viewDate) {
    historyDateEl.value = state.viewDate;
    setError("Save or cancel your edit before switching days");
    return;
  }
  if (state.saving || state.itemSaving || state.importing || state.viewDate === date) {
    historyDateEl.value = state.viewDate;
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
    if (request !== viewRequest || state.editing || state.itemSaving || state.saving) return;
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

async function saveCapture() {
  if (imageBusy()) return;
  const rawText = editorEl.value;
  if (!rawText.trim()) return;
  if (state.saving || state.itemSaving || state.importing) {
    return;
  }
  state.saving = true;
  ++viewRequest;
  updateCaptureControls();
  renderList();
  captureStatusEl.textContent = "";
  await refreshTodayIfStale();
  const trimmedText = rawText.replace(/\s+$/, "");

  try {
    const saved = await invoke("append_today_item", {
      text: trimmedText,
      id: await ensureDraftId(),
    });
    if (saved.date === state.todayDate) {
      state.todayItems = [...state.todayItems.filter(entry => entry.id !== saved.entry.id), saved.entry];
      syncTodayCount();
      if (state.viewDate === state.todayDate) state.viewItems = state.todayItems;
    } else {
      state.dayCounts.set(saved.date, saved.counts.current);
      if (state.viewDate === saved.date) state.viewItems = [...state.viewItems.filter(entry => entry.id !== saved.entry.id), saved.entry];
    }
    updateCount();
    renderList();
    renderDayStrip();
    clearEditor();
    setDrawerState(true);
    captureStatusEl.textContent = saved.date === state.todayDate ? 'Saved to Today' : `Saved to ${saved.date}`;
    setError(null);
  } catch (error) {
    console.error("append_today_item failed", error);
    captureStatusEl.textContent = `Save failed — your draft is still here. ${String(error.message || error)}`;
  } finally {
    state.saving = false;
    updateCaptureControls();
    renderList();
  }
}

async function handleFocus() {
  await refreshTodayIfStale();
}

async function boot() {
  try {
    editorEl.value = window.localStorage.getItem(DRAFT_KEY) || "";
    state.draftId = window.localStorage.getItem(DRAFT_ID_KEY) || null;
    renderPreviews(capturePreviewsEl, editorEl.value, state.draftId, invoke);
  } catch (error) {
    captureStatusEl.textContent = "Draft recovery unavailable. Save before quitting.";
  }
  updateCaptureControls();
  try {
    await loadTodayItems();
    await loadDayCounts();
    state.ready = true;
    updateCaptureControls();
    setDrawerState(false);
    focusEditor();
  } catch (error) {
    console.error("get_today_items failed", error);
    captureStatusEl.textContent = `Storage unavailable — ${String(error.message || error)}. Restart after resolving this.`;
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
  const joined = formatMarkdownChecklist(state.viewItems.map(entry => entry.text), {
    heading: todayHeading(),
  });
  copyText(joined);
});

historyDateEl.addEventListener("change", () => {
  if (historyDateEl.value) setViewDate(historyDateEl.value);
});
saveCaptureEl.addEventListener("click", saveCapture);
editorEl.addEventListener("input", () => {
  if (state.ready) captureStatusEl.textContent = "";
  persistDraft();
});
window.addEventListener("focus", handleFocus);
window.addEventListener("resize", renderDayStrip);
window.addEventListener("DOMContentLoaded", boot);
editorEl.addEventListener("keydown", (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
    event.preventDefault();
    saveCapture();
    return;
  }

});

const captureImages = installAttachmentInput(editorEl, {
  invoke, getId: ensureDraftId, isBusy: imageBusy, setBusy: setImageBusy,
  onChange: persistDraft, onStatus: message => {captureStatusEl.textContent = message;},
});
const capturePicker = attachmentPicker(document, captureImages.importFiles);
attachCaptureEl.after(capturePicker);
attachCaptureEl.addEventListener('click', () => capturePicker.click());
discardCaptureEl.addEventListener('click', discardCapture);
