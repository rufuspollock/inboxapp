import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { buildChangelog } from '../scripts/build-site-changelog.js';

async function fixture(t) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'inbox-changelog-'));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  await fs.mkdir(path.join(root, 'changelog/images'), { recursive: true });
  return root;
}

test('website renders dated snippets newest first with Markdown and local images', async t => {
  const root = await fixture(t);
  await fs.writeFile(path.join(root, 'changelog/2026-10-22-old.md'), '---\ndate: 2026-10-22\ntitle: Earlier entry\npromote: false\n---\n\nEarlier news.');
  await fs.writeFile(path.join(root, 'changelog/2026-10-25-new.md'), '---\ndate: 2026-10-25\ntitle: "Images & notes"\npromote: false\n---\n\n**Capture** images.\n\n![Preview](images/preview.png)\n\n[Download](https://example.com/download)');
  await fs.writeFile(path.join(root, 'changelog/images/preview.png'), 'image bytes');
  await buildChangelog(root);
  const html = await fs.readFile(path.join(root, 'site/changelog/index.html'), 'utf8');
  assert.ok(html.indexOf('Images &amp; notes') < html.indexOf('Earlier entry'));
  assert.match(html, /<strong>Capture<\/strong>/);
  assert.match(html, /src="images\/preview.png"/);
  assert.match(html, /href="https:\/\/example.com\/download"/);
  assert.match(html, /id="2026-10-25-new"/);
  assert.equal(await fs.readFile(path.join(root, 'site/changelog/images/preview.png'), 'utf8'), 'image bytes');
  await buildChangelog(root, { check: true });
});

test('freshness check fails after a snippet or screenshot changes', async t => {
  const root = await fixture(t);
  const source = path.join(root, 'changelog/2026-10-25-news.md');
  await fs.writeFile(source, '---\ndate: 2026-10-25\ntitle: News\npromote: false\n---\n\nOriginal.');
  await fs.writeFile(path.join(root, 'changelog/images/preview.png'), 'original');
  await buildChangelog(root);
  await fs.appendFile(source, '\nMore news.');
  await assert.rejects(buildChangelog(root, { check: true }), /build:site/);
  await buildChangelog(root);
  await fs.writeFile(path.join(root, 'changelog/images/preview.png'), 'updated');
  await assert.rejects(buildChangelog(root, { check: true }), /build:site/);
});

test('invalid frontmatter fails with the source filename', async t => {
  const root = await fixture(t);
  await fs.writeFile(path.join(root, 'changelog/2026-10-25-bad.md'), '---\ndate: yesterday\ntitle: Bad\n---\n\nNews.');
  await assert.rejects(buildChangelog(root), /2026-10-25-bad.md/);
});
