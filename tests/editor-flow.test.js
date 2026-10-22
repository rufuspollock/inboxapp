import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { JSDOM } from 'jsdom';

const pastDate = new Date();
pastDate.setDate(pastDate.getDate() - 1);
const past = `${pastDate.getFullYear()}-${String(pastDate.getMonth() + 1).padStart(2, '0')}-${String(pastDate.getDate()).padStart(2, '0')}`;
const html = await readFile(new URL('../src/index.html', import.meta.url), 'utf8');
let sequence = 0;
const settle = () => new Promise(resolve => setImmediate(resolve));
async function app({ draft, failSave = false, todayItems = [], pastItems = [], onInvoke } = {}) {
  const dom = new JSDOM(html, { url: 'http://inbox.local' });
  globalThis.window = dom.window;
  globalThis.document = dom.window.document;
  Object.defineProperty(document.querySelector('#day-strip'), 'clientWidth', { value: 760 });
  const calls = [];
  const now = new Date();
  const date = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
  if (draft) window.localStorage.setItem('inboxapp.capture-draft', draft);
  window.__TAURI__ = { core: { invoke: async (command, args) => {
    calls.push({ command, args });
    if (onInvoke) { const result = onInvoke(command, args); if (result !== undefined) return result; }
    if (command === 'get_today_items') return { filename: `${date}.md`, items: [...todayItems] };
    if (command === 'list_day_counts') return [{ date: past, count: pastItems.length }];
    if (command === 'get_items_for_date') return { date: args.date, items: [...pastItems] };
    if (failSave && ['append_today_item', 'update_item_for_date'].includes(command)) throw new Error('Disk full');
    if (command === 'update_item_for_date') return { date: args.date, items: pastItems.map((item, index) => index === args.index ? args.item : item) };
    return {};
  } } };
  await import(`../src/main.js?test=${++sequence}`);
  window.dispatchEvent(new window.Event('DOMContentLoaded'));
  await settle();
  const editor = document.querySelector('#editor');
  function type(element, text) {
    element.value = text;
    element.dispatchEvent(new window.Event('input', { bubbles: true }));
  }
  return { dom, calls, editor, type, date };
}

test('draft survives blur and focus without writing a journal entry', async () => {
  const { editor, type, calls } = await app();
  type(editor, 'Keep this thought\n\nAnd its second paragraph');
  window.dispatchEvent(new window.Event('blur'));
  await settle();
  window.dispatchEvent(new window.Event('focus'));
  await settle();
  assert.equal(editor.value, 'Keep this thought\n\nAnd its second paragraph');
  assert.equal(window.localStorage.getItem('inboxapp.capture-draft'), editor.value);
  assert.equal(calls.filter(c => c.command === 'append_today_item').length, 0);
});

test('restart restores draft and explicit save appends exactly once then clears it', async () => {
  const { editor, calls } = await app({ draft: 'Recovered draft' });
  assert.equal(editor.value, 'Recovered draft');
  const save = document.querySelector('#save-capture');
  assert.ok(save, 'Save button is available');
  save.click();
  save.click();
  await settle();
  assert.deepEqual(calls.filter(c => c.command === 'append_today_item').map(c => c.args.text), ['Recovered draft']);
  assert.equal(editor.value, '');
  assert.equal(window.localStorage.getItem('inboxapp.capture-draft'), null);
});

test('failed explicit save keeps the draft available for retry', async (t) => {
  t.mock.method(console, 'error', () => {});
  const { editor, type } = await app({ failSave: true });
  type(editor, 'Do not lose me');
  const save = document.querySelector('#save-capture');
  assert.ok(save);
  save.click();
  await settle();
  assert.equal(editor.value, 'Do not lose me');
  assert.match(document.querySelector('#capture-status').textContent, /failed/i);
  assert.equal(save.disabled, false);
});

test('Command+Enter explicitly saves while ordinary Enter remains multiline', async () => {
  const { editor, type, calls } = await app();
  type(editor, 'Keyboard capture');
  editor.dispatchEvent(new window.KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
  await settle();
  assert.equal(calls.filter(c => c.command === 'append_today_item').length, 0);
  editor.dispatchEvent(new window.KeyboardEvent('keydown', { key: 'Enter', metaKey: true, bubbles: true }));
  await settle();
  assert.equal(calls.filter(c => c.command === 'append_today_item').length, 1);
});

async function openPast(options = {}) {
  const fixture = await app({ pastItems: ['- [x] Original\n\nSecond paragraph', 'Other entry'], ...options });
  const picker = document.querySelector('#history-date');
  picker.value = past;
  picker.dispatchEvent(new window.Event('change')); 
  await settle();
  const text = [...document.querySelectorAll('.drawer__item-text')].find(el => el.textContent.includes('Original'));
  text.click();
  await settle();
  return fixture;
}

test('editing an old entry preserves date, checked state and multiline text', async () => {
  const { calls, type } = await openPast();
  const edit = document.querySelector('.drawer__item-editor');
  assert.ok(edit, 'Clicking an entry opens its editor');
  assert.equal(edit.value, 'Original\n\nSecond paragraph');
  type(edit, 'Changed\n\nStill multiline');
  document.querySelector('.drawer__item-save').click();
  await settle();
  const update = calls.find(c => c.command === 'update_item_for_date');
  assert.deepEqual(update.args, { date: past, index: 0, item: '- [x] Changed\n\nStill multiline', expectedItem: '- [x] Original\n\nSecond paragraph' });
  assert.equal(document.querySelector('.drawer__item-editor'), null);
});

test('cancelling an edit leaves the saved old entry untouched', async () => {
  const { calls, type } = await openPast();
  const edit = document.querySelector('.drawer__item-editor');
  assert.ok(edit);
  type(edit, 'Discard this edit');
  document.querySelector('.drawer__item-cancel').click();
  await settle();
  assert.equal(calls.filter(c => c.command === 'update_item_for_date').length, 0);
  assert.match(document.querySelector('#today-list').textContent, /Original/);
});


test('day navigation keeps an unfinished edit visible', async () => {
  const { date, type } = await openPast();
  type(document.querySelector('.drawer__item-editor'), 'Unfinished');
  document.querySelector(`[data-date="${date}"]`).click();
  await settle();
  assert.equal(document.querySelector('.drawer__item-editor').value, 'Unfinished');
  assert.match(document.querySelector('#drawer-status').textContent, /save or cancel/i);
});

test('failed historical save retains editable text and original entry', async (t) => {
  t.mock.method(console, 'error', () => {});
  const { type } = await openPast({ failSave: true });
  type(document.querySelector('.drawer__item-editor'), 'Keep my correction');
  document.querySelector('.drawer__item-save').click();
  await settle();
  assert.equal(document.querySelector('.drawer__item-editor').value, 'Keep my correction');
  assert.match(document.querySelector('#drawer-status').textContent, /save failed/i);
  document.querySelector('.drawer__item-cancel').click();
  assert.match(document.querySelector('#today-list').textContent, /Original/);
});


test('a pending deletion prevents starting an edit or capture save', async () => {
  let finishDelete;
  const pendingDelete = new Promise(resolve => { finishDelete = resolve; });
  const { type, editor, calls } = await app({ todayItems: ['First', 'Second'], onInvoke: command => command === 'delete_item_for_date' ? pendingDelete : undefined });
  type(editor, 'New draft');
  document.querySelector('.drawer__item-delete').click();
  document.querySelector('.drawer__item-text').click();
  document.querySelector('#save-capture').click();
  assert.equal(document.querySelector('.drawer__item-editor'), null);
  assert.equal(calls.filter(c => c.command === 'append_today_item').length, 0);
  finishDelete({ items: ['First'] });
  await settle();
  document.querySelector('.drawer__item-text').click();
  assert.equal(document.querySelector('.drawer__item-editor').value, 'First');
});

test('returning to Today cancels a pending historical load', async () => {
  let finishRead;
  const pending = new Promise(resolve => { finishRead = resolve; });
  const { date } = await app({ todayItems: ['Today entry'], onInvoke: command => command === 'get_items_for_date' ? pending : undefined });
  const picker = document.querySelector('#history-date');
  picker.value = past;
  picker.dispatchEvent(new window.Event('change'));
  document.querySelector(`[data-date="${date}"]`).click();
  finishRead({ date: past, items: ['Stale day'] });
  await settle();
  assert.equal(picker.value, date);
  assert.match(document.querySelector('#today-list').textContent, /Today entry/);
});

test('a day rollover preserves the date and text of an unfinished edit', async () => {
  let reads = 0;
  const { type } = await app({ todayItems: ['New day'], onInvoke: command => {
    if (command === 'get_today_items' && reads++ === 0) return { filename: `${past}.md`, items: ['Yesterday draft'] };
  } });
  document.querySelector('.drawer__item-text').click();
  type(document.querySelector('.drawer__item-editor'), 'Still editing yesterday');
  window.dispatchEvent(new window.Event('focus'));
  await settle();
  assert.equal(document.querySelector('#history-date').value, past);
  assert.equal(document.querySelector('.drawer__item-editor').value, 'Still editing yesterday');
});
