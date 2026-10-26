const MAX_FILE_BYTES = 20 * 1024 * 1024;

function attachmentMarkdown(file) {
  const label = file.name.replace(/[\\\[\]]/g, '\\$&').replace(/[\r\n]/g, ' ');
  return `${file.is_image ? '!' : ''}[${label}](${file.path})`;
}

// Import first, then insert a link: failed writes never leave broken Markdown.
export function installAttachmentInput(field, {invoke, getId, isBusy, setBusy, onChange, onStatus}) {
  async function insert(load) {
    if (isBusy()) return;
    const start = field.selectionStart;
    const end = field.selectionEnd;
    const original = field.value;
    setBusy(true);
    onStatus('Adding attachment…');
    try {
      const {text, error} = await load();
      onStatus('');
      if (text) {
        field.value = original.slice(0, start) + text + original.slice(end);
        field.setSelectionRange(start + text.length, start + text.length);
        onChange();
      }
      if (error) onStatus(error);
    } catch (error) {
      onStatus(`Paste/import failed — ${String(error.message || error)}`);
    } finally {
      setBusy(false);
      field.focus();
    }
  }
  async function paste() {
    return insert(async () => {
      const result = await invoke('paste_attachments', {id: await getId()});
      return {text: result.attachments.length ? result.attachments.map(attachmentMarkdown).join('\n') : result.text, error: result.error};
    });
  }
  async function importFile(file) {
    return importFiles(file ? [file] : []);
  }
  async function importFiles(files) {
    if (!files.length || isBusy()) return;
    return insert(async () => {
      let id;
      const links = [], errors = [];
      for (const file of files) {
        try {
          if (file.size > MAX_FILE_BYTES) throw new Error('File exceeds the 20 MB limit.');
          id ||= await getId();
          const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
          const imported = await invoke('import_file', {id, name:file.name || 'attachment', bytes});
          links.push(attachmentMarkdown(imported));
        } catch (error) { errors.push(`${file.name || 'File'}: ${String(error.message || error)}`); }
      }
      return {text:links.join('\n'), error:errors.join(' ')};
    });
  }
  field.addEventListener('paste', event => {
    if (isBusy()) {event.preventDefault(); return;}
    const files = Array.from(event.clipboardData?.files || []);
    event.preventDefault();
    if (files.length) importFiles(files);
    else paste();
  });
  field.addEventListener('keydown', event => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'v') {
      event.preventDefault();
      event.stopPropagation();
      paste();
    }
  });
  return {paste, importFile, importFiles};
}

function attachments(text, id) {
  if (!id) return [];
  const files = [];
  for (const match of text.matchAll(/(!?)\[((?:\\.|[^\]\\\n])*)\]\((assets\/[a-z0-9-]+\/[a-zA-Z0-9._-]+)\)/g)) {
    const path = match[3];
    if (path.startsWith(`assets/${id}/`) && !files.some(file => file.path === path)) {
      files.push({path, name:match[2].replace(/\\(.)/g, '$1') || path.split('/').at(-1), is_image:!!match[1]});
    }
  }
  return files;
}

export function attachmentPaths(text, id) {
  return attachments(text, id).map(file => file.path);
}

export async function renderPreviews(container, text, id, invoke) {
  const files = attachments(text, id);
  const signature = JSON.stringify(files);
  if (container.previewSignature === signature) return;
  container.previewSignature = signature;
  const revision = {};
  container.previewRevision = revision;
  container.replaceChildren();
  for (const {path, name, is_image} of files) {
    const document = container.ownerDocument;
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'attachment-preview';
    button.title = is_image ? 'View image' : `Open ${name}`;
    button.setAttribute('aria-label', is_image ? 'View attached image' : `Open attached file ${name}`);
    container.append(button);
    if (!is_image) {
      button.classList.add('attachment-file');
      button.textContent = name;
      button.addEventListener('click', async () => {
        button.disabled = true;
        try {
          await invoke('open_attachment', {id, path});
          button.textContent = name;
        } catch (error) {
          button.textContent = `Could not open ${name}`;
          button.title = String(error.message || error);
        } finally { button.disabled = false; }
      });
      continue;
    }
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

export function attachmentPicker(document, onFile) {
  const input = document.createElement('input');
  input.type = 'file';
  input.multiple = true;
  input.hidden = true;
  input.addEventListener('change', () => {onFile(Array.from(input.files || [])); input.value = '';});
  return input;
}
