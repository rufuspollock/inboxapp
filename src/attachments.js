const MAX_IMAGE_BYTES = 20 * 1024 * 1024;
const IMAGE_TYPES = new Set(['image/png', 'image/jpeg', 'image/webp']);

// Import first, then insert a link: failed writes never leave broken Markdown.
export function installImageInput(field, {invoke, getId, isBusy, setBusy, onChange, onStatus}) {
  async function insert(load) {
    if (isBusy()) return;
    const start = field.selectionStart;
    const end = field.selectionEnd;
    const original = field.value;
    setBusy(true);
    onStatus('Adding image…');
    try {
      const text = await load();
      onStatus('');
      if (text) {
        field.value = original.slice(0, start) + text + original.slice(end);
        field.setSelectionRange(start + text.length, start + text.length);
        onChange();
      }
    } catch (error) {
      onStatus(`Paste/import failed — ${String(error.message || error)}`);
    } finally {
      setBusy(false);
      field.focus();
    }
  }
  async function paste() {
    return insert(async () => {
      const path = await invoke('paste_image', {id: await getId()});
      if (path) return `![Image](${path})`;
      return await invoke('plugin:clipboard-manager|read_text') || '';
    });
  }
  async function importFile(file) {
    if (!file || isBusy()) return;
    if (!IMAGE_TYPES.has(file.type)) {
      onStatus('Choose a PNG, JPEG, or WebP image.');
      return;
    }
    if (file.size > MAX_IMAGE_BYTES) {
      onStatus('Image is too large. Maximum size is 20 MB.');
      return;
    }
    return insert(async () => {
      const id = await getId();
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      const path = await invoke('import_attachment', {id, bytes});
      return `![Image](${path})`;
    });
  }
  field.addEventListener('paste', event => {
    if (isBusy()) {event.preventDefault(); return;}
    const files = Array.from(event.clipboardData?.files || []);
    if (files.length) {
      event.preventDefault();
      if (files.length > 1) onStatus('Paste one image at a time.');
      else importFile(files[0]);
    } else if (!event.clipboardData?.getData('text/plain')) {
      // macOS Edit > Paste may expose image bytes only through the native clipboard.
      event.preventDefault();
      paste();
    }
    // Ordinary text keeps the textarea's normal paste behavior (and input event).
  });
  field.addEventListener('keydown', event => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'v') {
      event.preventDefault();
      event.stopPropagation();
      paste();
    }
  });
  return {paste, importFile};
}

export function attachmentPaths(text, id) {
  if (!id) return [];
  const paths = [];
  for (const match of text.matchAll(/!\[[^\]\n]*\]\((assets\/[a-z0-9-]+\/[a-zA-Z0-9._-]+)\)/g)) {
    if (match[1].startsWith(`assets/${id}/`) && !paths.includes(match[1])) paths.push(match[1]);
  }
  return paths;
}

export async function renderPreviews(container, text, id, invoke) {
  const paths = attachmentPaths(text, id);
  const signature = JSON.stringify(paths);
  if (container.previewSignature === signature) return;
  container.previewSignature = signature;
  const revision = {};
  container.previewRevision = revision;
  container.replaceChildren();
  for (const path of paths) {
    const document = container.ownerDocument;
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'attachment-preview';
    button.title = 'View image';
    button.setAttribute('aria-label', 'View attached image');
    container.append(button);
    try {
      const url = await invoke('read_attachment', {id, path});
      if (container.previewRevision !== revision) return;
      const img = document.createElement('img');
      img.alt = 'Attached image';
      img.src = url;
      button.append(img);
      button.addEventListener('click', () => {
        const dialog = document.createElement('dialog');
        dialog.className = 'image-viewer';
        const large = img.cloneNode();
        const close = document.createElement('button');
        close.textContent = 'Close';
        close.type = 'button';
        close.addEventListener('click', () => dialog.close());
        dialog.addEventListener('close', () => dialog.remove());
        dialog.addEventListener('click', event => {if (event.target === dialog) dialog.close();});
        dialog.append(large, close);
        document.body.append(dialog);
        dialog.showModal();
      });
    } catch {
      button.textContent = 'Image unavailable';
      button.disabled = true;
    }
  }
}

export function imagePicker(document, onFile) {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = 'image/png,image/jpeg,image/webp';
  input.hidden = true;
  input.addEventListener('change', () => {onFile(input.files?.[0]); input.value = '';});
  return input;
}
