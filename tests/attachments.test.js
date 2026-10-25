import test from 'node:test';
import assert from 'node:assert/strict';
import { JSDOM } from 'jsdom';
import { installImageInput, attachmentPaths, renderPreviews } from '../src/attachments.js';

function fixture(invoke) {
  const dom = new JSDOM('<textarea></textarea><div id="previews"></div>');
  const field = dom.window.document.querySelector('textarea');
  field.value = 'Before after';
  field.setSelectionRange(7, 7);
  let busy = false;
  const statuses = [];
  let changes = 0;
  const controls = installImageInput(field, {
    invoke, getId: async () => '2026-10-25-0001', isBusy: () => busy,
    setBusy: value => { busy = value; field.readOnly = value; },
    onChange: () => changes++, onStatus: text => statuses.push(text),
  });
  return {dom, field, controls, statuses, changes: () => changes, busy: () => busy};
}

test('image paste inserts a portable link at the cursor after durable import', async () => {
  const calls = [];
  const f = fixture(async (command, args) => {
    calls.push({command, args});
    if (command === 'paste_image') return 'assets/2026-10-25-0001/image-1.png';
  });
  await f.controls.paste();
  assert.equal(f.field.value, 'Before ![Image](assets/2026-10-25-0001/image-1.png)after');
  assert.equal(f.changes(), 1);
  assert.equal(f.busy(), false);
  assert.equal(calls[0].args.id, '2026-10-25-0001');
});

test('ordinary clipboard text remains text and replaces selected text', async () => {
  const f = fixture(async command => command === 'paste_image' ? null : 'plain text');
  f.field.setSelectionRange(0, 6);
  await f.controls.paste();
  assert.equal(f.field.value, 'plain text after');
});

test('failed image import preserves text and selection without a broken link', async () => {
  const f = fixture(async () => { throw new Error('Disk full'); });
  await f.controls.paste();
  assert.equal(f.field.value, 'Before after');
  assert.equal(f.field.selectionStart, 7);
  assert.equal(f.changes(), 0);
  assert.match(f.statuses.at(-1), /failed/i);
  assert.equal(f.busy(), false);
});

test('pending image import blocks repeated paste until complete', async () => {
  let done;
  let calls = 0;
  const f = fixture(async () => {calls++; return new Promise(resolve => {done = resolve;});});
  const pending = f.controls.paste();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(f.busy(), true);
  await f.controls.paste();
  assert.equal(calls, 1);
  done('assets/2026-10-25-0001/image-1.png');
  await pending;
});

test('image picker validates size/type before reading bytes', async () => {
  const f = fixture(async () => {throw new Error('must not import');});
  await f.controls.importFile({type: 'text/html', size: 1});
  assert.match(f.statuses.at(-1), /PNG.*JPEG.*WebP/);
  await f.controls.importFile({type: 'image/png', size: 21 * 1024 * 1024});
  assert.match(f.statuses.at(-1), /20 MB/);
});

test('previews only read images owned by the entry; external URLs stay inert', async () => {
  const text = '![Image](assets/2026-10-25-0001/image-1.png) ![x](https://example.com/x.png) ![x](assets/2026-10-25-0002/other.png)';
  assert.deepEqual(attachmentPaths(text, '2026-10-25-0001'), ['assets/2026-10-25-0001/image-1.png']);
  const f = fixture(async () => {});
  const container = f.dom.window.document.querySelector('#previews');
  const calls = [];
  await renderPreviews(container, text, '2026-10-25-0001', async (command, args) => {calls.push({command,args}); return 'data:image/png;base64,aGVsbG8=';});
  assert.equal(container.querySelectorAll('img').length, 1);
  assert.equal(calls[0].command, 'read_attachment');
});

test('menu Paste falls back to native image import when WebKit exposes no clipboard files', async () => {
  const f = fixture(async command => command === 'paste_image' ? 'assets/2026-10-25-0001/image-1.png' : '');
  const event = new f.dom.window.Event('paste', {cancelable:true});
  Object.defineProperty(event, 'clipboardData', {value:{files:[], getData:()=>''}});
  f.field.dispatchEvent(event);
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(event.defaultPrevented, true);
  assert.match(f.field.value, /!\[Image\]/);
});

test('typing beside unchanged image links does not repeatedly decode the same previews', async () => {
  const f = fixture(async () => {});
  const container = f.dom.window.document.querySelector('#previews');
  let reads = 0;
  const invoke = async () => {reads++; return 'data:image/png;base64,aGVsbG8=';};
  const link = '![Image](assets/2026-10-25-0001/image-1.png)';
  await renderPreviews(container, link, '2026-10-25-0001', invoke);
  await renderPreviews(container, 'More text ' + link, '2026-10-25-0001', invoke);
  assert.equal(reads, 1);
  await renderPreviews(container, 'Removed link', '2026-10-25-0001', invoke);
  assert.equal(container.querySelectorAll('img').length, 0);
});
