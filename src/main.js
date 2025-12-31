const { invoke } = window.__TAURI__.core;

const state = {
  filename: "",
  saveTimer: null,
};

const editorEl = document.querySelector("#editor");

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
    moveCursorToEnd();
  } catch (error) {
    console.error("get_active_file failed", error);
  }
}

editorEl.addEventListener("input", scheduleSave);

window.addEventListener("DOMContentLoaded", boot);
