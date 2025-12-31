const { invoke } = window.__TAURI__.core;

const state = {
  filename: "",
  saveTimer: null,
  showArchived: false,
  activeLines: [],
  archivedLines: [],
};

const editorEl = document.querySelector("#editor");
const renderEl = document.querySelector("#render");
const toggleArchiveEl = document.querySelector("#toggle-archive");

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function formatInline(text) {
  let html = escapeHtml(text);
  html = html.replace(/\*\*([^*]+)\*\*/g, "<span class=\"render__bold\">**$1**</span>");
  html = html.replace(/\*([^*]+)\*/g, "<span class=\"render__italic\">*$1*</span>");
  return html;
}

function splitArchived(text) {
  const lines = text.split("\n");
  const archivedIndex = lines.findIndex(
    (line) => line.trimEnd() === "## Archived"
  );
  if (archivedIndex === -1) {
    return { active: lines, archived: [] };
  }
  return {
    active: lines.slice(0, archivedIndex),
    archived: lines.slice(archivedIndex + 1),
  };
}

function renderLine(line, section, lineIndex) {
  if (section === "archived") {
    const content = formatInline(line);
    return `
      <div class="render__line render__line--archived" data-section="${section}" data-line-index="${lineIndex}">
        <span class="render__text">${content || "&nbsp;"}</span>
        <button class="render__restore" data-action="restore" type="button">Restore</button>
      </div>
    `;
  }

  const checklistMatch = line.match(/^(\s*)- \[( |x|X)\]\s?(.*)$/);
  if (checklistMatch) {
    const [, indent, checkedToken, content] = checklistMatch;
    const checked = checkedToken.toLowerCase() === "x";
    const text = formatInline(`${indent}${content}`);
    const checkedClass = checked ? " render__checkbox--checked" : "";
    return `
      <div class="render__line render__line--checklist" data-section="${section}" data-line-index="${lineIndex}">
        <button class="render__checkbox${checkedClass}" data-action="archive" type="button" aria-label="Archive item"></button>
        <span class="render__text">${text || "&nbsp;"}</span>
      </div>
    `;
  }

  const content = formatInline(line);
  return `
    <div class="render__line" data-section="${section}" data-line-index="${lineIndex}">
      <span class="render__text">${content || "&nbsp;"}</span>
    </div>
  `;
}

function renderSection(lines, section) {
  return lines
    .map((line, index) => renderLine(line, section, index))
    .join("");
}

function renderEditor(text) {
  const { active, archived } = splitArchived(text);
  state.activeLines = active;
  state.archivedLines = archived;

  let html = renderSection(active, "active");
  if (state.showArchived && archived.length > 0) {
    html += `
      <div class="render__heading">## Archived</div>
      ${renderSection(archived, "archived")}
    `;
  }

  renderEl.innerHTML = html;
}

function scheduleSave() {
  if (state.saveTimer) {
    clearTimeout(state.saveTimer);
  }
  state.saveTimer = setTimeout(saveNow, 400);
}

async function saveNow() {
  const text = editorEl.value;
  try {
    await invoke("save_active_file", {
      filename: state.filename,
      text,
    });
  } catch (error) {
    console.error("save_active_file failed", error);
  }
}

async function archiveLine(lineIndex) {
  const lineText = state.activeLines[lineIndex];
  if (lineText === undefined) {
    return;
  }
  try {
    const result = await invoke("archive_item", {
      filename: state.filename,
      lineIdx: lineIndex,
      lineText,
    });
    editorEl.value = result.text;
    renderEditor(result.text);
  } catch (error) {
    console.error("archive_item failed", error);
  }
}

async function restoreLine(lineIndex) {
  const lineText = state.archivedLines[lineIndex];
  if (lineText === undefined) {
    return;
  }
  try {
    const result = await invoke("restore_item", {
      filename: state.filename,
      lineIdx: lineIndex,
      lineText,
    });
    editorEl.value = result.text;
    renderEditor(result.text);
  } catch (error) {
    console.error("restore_item failed", error);
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
    editorEl.value = data.text;
    renderEditor(data.text);
    moveCursorToEnd();
  } catch (error) {
    console.error("get_active_file failed", error);
  }
}

editorEl.addEventListener("input", () => {
  scheduleSave();
  renderEditor(editorEl.value);
});

editorEl.addEventListener("scroll", () => {
  renderEl.scrollTop = editorEl.scrollTop;
  renderEl.scrollLeft = editorEl.scrollLeft;
});

renderEl.addEventListener("click", (event) => {
  const actionTarget = event.target.closest("[data-action]");
  if (!actionTarget) {
    editorEl.focus();
    return;
  }
  const lineEl = actionTarget.closest(".render__line");
  if (!lineEl) {
    editorEl.focus();
    return;
  }

  const lineIndex = Number(lineEl.dataset.lineIndex || 0);
  if (actionTarget.dataset.action === "archive") {
    archiveLine(lineIndex);
    return;
  }
  if (actionTarget.dataset.action === "restore") {
    restoreLine(lineIndex);
    return;
  }
});

toggleArchiveEl.addEventListener("click", () => {
  state.showArchived = !state.showArchived;
  toggleArchiveEl.textContent = state.showArchived
    ? "Hide archived"
    : "Show archived";
  renderEditor(editorEl.value);
});

window.addEventListener("DOMContentLoaded", boot);
