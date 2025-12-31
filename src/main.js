const { invoke } = window.__TAURI__.core;

const state = {
  filename: "",
  archivedLines: [],
  saveTimer: null,
};

const countsEl = document.querySelector("#counts");
const editorEl = document.querySelector("#editor");
const listEl = document.querySelector("#list");

function splitArchived(text) {
  const lines = text.split(/\r?\n/);
  const active = [];
  const archived = [];
  let inArchived = false;

  for (const line of lines) {
    if (line.trim() === "## Archived") {
      inArchived = true;
      continue;
    }
    if (inArchived) {
      archived.push(line);
    } else {
      active.push(line);
    }
  }

  return {
    activeText: active.join("\n").replace(/\s+$/, ""),
    archivedLines: archived,
  };
}

function combineWithArchived(activeText, archivedLines) {
  const trimmed = activeText.replace(/\s+$/, "");
  if (!archivedLines.length || archivedLines.every((line) => !line.trim())) {
    return trimmed;
  }

  const archivedText = archivedLines.join("\n");
  if (!trimmed) {
    return `## Archived\n${archivedText}`;
  }
  return `${trimmed}\n\n## Archived\n${archivedText}`;
}

function parseActiveLines(activeText) {
  return activeText
    .split(/\r?\n/)
    .filter((line) => line.trim().length > 0);
}

function renderList(activeText) {
  const lines = parseActiveLines(activeText);
  listEl.innerHTML = "";

  lines.forEach((line, idx) => {
    const item = document.createElement("label");
    item.className = "list-item";

    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.className = "list-item__checkbox";

    const text = document.createElement("span");
    text.className = "list-item__text";
    text.textContent = line.replace(/^[-*+]\s+/, "");

    checkbox.addEventListener("change", async () => {
      checkbox.disabled = true;
      try {
        const result = await invoke("archive_item", {
          filename: state.filename,
          line_idx: idx,
        });
        const split = splitArchived(result.text);
        state.archivedLines = split.archivedLines;
        editorEl.value = split.activeText;
        updateCounts(result.counts);
        renderList(split.activeText);
      } catch (error) {
        console.error("archive_item failed", error);
        checkbox.checked = false;
        checkbox.disabled = false;
      }
    });

    item.appendChild(checkbox);
    item.appendChild(text);
    listEl.appendChild(item);
  });
}

function updateCounts(counts) {
  countsEl.textContent = `Current ${counts.current} · Total ${counts.total} · Files ${counts.files}`;
}

function scheduleSave() {
  if (state.saveTimer) {
    clearTimeout(state.saveTimer);
  }
  state.saveTimer = setTimeout(saveNow, 400);
}

async function saveNow() {
  const activeText = editorEl.value;
  const combined = combineWithArchived(activeText, state.archivedLines);
  try {
    const counts = await invoke("save_active_file", {
      filename: state.filename,
      text: combined,
    });
    updateCounts(counts);
  } catch (error) {
    console.error("save_active_file failed", error);
  }
}

function moveCursorToEnd() {
  const length = editorEl.value.length;
  editorEl.focus();
  editorEl.setSelectionRange(length, length);
}

async function boot() {
  try {
    const data = await invoke("get_active_file");
    state.filename = data.filename;
    const split = splitArchived(data.text);
    state.archivedLines = split.archivedLines;
    editorEl.value = split.activeText;
    updateCounts(data.counts);
    renderList(split.activeText);
    moveCursorToEnd();
  } catch (error) {
    console.error("get_active_file failed", error);
  }
}

editorEl.addEventListener("input", () => {
  renderList(editorEl.value);
  scheduleSave();
});

window.addEventListener("DOMContentLoaded", boot);
