# InboxApp Landing Page Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a minimalist, accessible landing page in `site/index.html` using Tailwind CDN that reflects the app's palette, typography, and messaging.

**Architecture:** A single static HTML file with Tailwind via CDN, a small embedded `<style>` block for the app color tokens and monospace font stack, and semantic sections for hero, features, demo video, and download CTA.

**Tech Stack:** HTML, Tailwind CSS (CDN), minimal inline CSS for custom variables.

### Task 1: Create the `site/` directory and base HTML scaffold

**Files:**
- Create: `site/index.html`

**Step 1: Add the document skeleton**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Inbox — Just capture. Nothing else.</title>
    <script src="https://cdn.tailwindcss.com"></script>
  </head>
  <body>
  </body>
</html>
```

**Step 2: Add a `<style>` block for shared tokens**

```html
<style>
  :root {
    --bg-start: #f8f4ee;
    --bg-end: #efe7dc;
    --ink: #14120f;
    --muted-ink: #6b635b;
    --divider: #e1d7ca;
    --accent: #c9b59c;
    --selection: #e7ddcf;
    --day-2: #bdd9b3;
    --day-3: #98c58f;
  }

  body {
    font-family: "Iosevka", "IBM Plex Mono", "SFMono-Regular", "SF Mono",
      "Menlo", "Monaco", "Courier New", monospace;
    background: radial-gradient(circle at top, #faf7f1 0%, var(--bg-start) 40%, var(--bg-end) 100%);
    color: var(--ink);
  }

  ::selection {
    background: var(--selection);
  }
</style>
```

**Step 3: Add the top-level layout container**

```html
<body class="min-h-screen">
  <main class="mx-auto flex w-full max-w-5xl flex-col gap-16 px-6 pb-20 pt-16 sm:px-10">
    <!-- Sections go here -->
  </main>
</body>
```

### Task 2: Build hero section with logo-inspired mark and CTAs

**Files:**
- Modify: `site/index.html`

**Step 1: Add hero markup**

```html
<section class="flex flex-col gap-10">
  <div class="flex flex-col gap-6 sm:flex-row sm:items-center sm:justify-between">
    <div class="flex max-w-2xl flex-col gap-5">
      <div class="flex items-center gap-3">
        <!-- Inline SVG mark here -->
        <span class="text-sm uppercase tracking-[0.3em] text-[color:var(--muted-ink)]">Inbox</span>
      </div>
      <h1 class="text-3xl font-semibold leading-tight sm:text-5xl">Just capture. Nothing else.</h1>
      <p class="text-base leading-relaxed text-[color:var(--muted-ink)] sm:text-lg">
        A hyper-minimal macOS inbox for fleeting thoughts. One surface, raw Markdown, zero friction.
      </p>
      <div class="flex flex-wrap gap-3">
        <a class="rounded-full border border-transparent bg-[color:var(--ink)] px-5 py-2 text-sm text-[color:var(--bg-start)] transition hover:translate-y-[-1px] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2" href="https://github.com/<org>/<repo>/releases">
          Download (macOS)
        </a>
        <a class="rounded-full border border-[color:var(--divider)] px-5 py-2 text-sm text-[color:var(--ink)] transition hover:border-[color:var(--accent)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2" href="#demo">
          Watch demo
        </a>
      </div>
    </div>
    <div class="rounded-3xl border border-[color:var(--divider)] bg-white/40 p-6 shadow-[0_10px_40px_rgba(20,18,15,0.08)]">
      <!-- Minimalist mockup block -->
    </div>
  </div>
</section>
```

**Step 2: Add a simple inline SVG mark**

```html
<svg class="h-12 w-12" viewBox="0 0 64 64" aria-hidden="true">
  <defs>
    <linearGradient id="tray" x1="0" x2="1">
      <stop offset="0" stop-color="#bcd4f0" />
      <stop offset="1" stop-color="#6aa7df" />
    </linearGradient>
  </defs>
  <rect x="10" y="24" width="44" height="28" rx="6" fill="url(#tray)" />
  <path d="M16 28h32v10c0 4-3 7-7 7H23c-4 0-7-3-7-7V28z" fill="#ffffff" opacity="0.8" />
  <path d="M32 12l8 8h-5v8h-6v-8h-5l8-8z" fill="#e15a4f" />
</svg>
```

**Step 3: Add the hero mockup block**

```html
<div class="flex flex-col gap-4 text-xs text-[color:var(--muted-ink)]">
  <div class="flex items-center justify-between rounded-full border border-[color:var(--divider)] px-4 py-2">
    <span>Inbox — 7</span>
    <span>Today</span>
  </div>
  <div class="h-40 rounded-2xl border border-dashed border-[color:var(--divider)] bg-white/60 p-4">
    <p class="text-[color:var(--ink)]">• Capture thought</p>
    <p>• Another idea</p>
  </div>
  <div class="flex gap-2">
    <span class="h-2 w-6 rounded-full bg-[color:var(--day-2)]"></span>
    <span class="h-2 w-4 rounded-full bg-[color:var(--day-3)]"></span>
  </div>
</div>
```

### Task 3: Add feature and philosophy sections

**Files:**
- Modify: `site/index.html`

**Step 1: Add a "Why Inbox" grid**

```html
<section class="grid gap-6 border-t border-[color:var(--divider)] pt-10 sm:grid-cols-2">
  <h2 class="text-xl font-semibold">Why Inbox</h2>
  <ul class="flex flex-col gap-4 text-sm text-[color:var(--muted-ink)]">
    <li>Capture in under a second without derailing focus.</li>
    <li>One surface, no tabs, no navigation, no decisions.</li>
    <li>Raw Markdown saved locally in `~/.inboxapp`.</li>
    <li>No AI, no tags, no organization — just capture.</li>
  </ul>
</section>
```

**Step 2: Add "How it works" section**

```html
<section class="grid gap-6 sm:grid-cols-3">
  <div class="rounded-2xl border border-[color:var(--divider)] p-5">
    <h3 class="text-sm font-semibold">Open</h3>
    <p class="mt-2 text-sm text-[color:var(--muted-ink)]">Cursor lands at the end of today’s file.</p>
  </div>
  <div class="rounded-2xl border border-[color:var(--divider)] p-5">
    <h3 class="text-sm font-semibold">Type</h3>
    <p class="mt-2 text-sm text-[color:var(--muted-ink)]">Append-only Markdown, timestamps optional.</p>
  </div>
  <div class="rounded-2xl border border-[color:var(--divider)] p-5">
    <h3 class="text-sm font-semibold">Process</h3>
    <p class="mt-2 text-sm text-[color:var(--muted-ink)]">Move items elsewhere until the count hits zero.</p>
  </div>
</section>
```

### Task 4: Embed demo video and add download CTA

**Files:**
- Modify: `site/index.html`

**Step 1: Add demo section**

```html
<section id="demo" class="flex flex-col gap-6">
  <h2 class="text-xl font-semibold">Demo</h2>
  <div class="aspect-video w-full overflow-hidden rounded-2xl border border-[color:var(--divider)]">
    <iframe
      class="h-full w-full"
      src="https://www.youtube.com/embed/JdY6tsNhN30"
      title="InboxApp demo"
      allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
      allowfullscreen
    ></iframe>
  </div>
</section>
```

**Step 2: Add download + status callout**

```html
<section class="flex flex-col gap-4 rounded-3xl border border-[color:var(--divider)] bg-white/60 p-8">
  <h2 class="text-xl font-semibold">Download the alpha</h2>
  <p class="text-sm text-[color:var(--muted-ink)]">
    Developer-only for now. Packaged installer coming later. Feedback welcome.
  </p>
  <div class="flex flex-wrap gap-3">
    <a class="rounded-full border border-transparent bg-[color:var(--ink)] px-5 py-2 text-sm text-[color:var(--bg-start)]" href="https://github.com/<org>/<repo>/releases">
      Get the latest build
    </a>
    <a class="rounded-full border border-[color:var(--divider)] px-5 py-2 text-sm text-[color:var(--ink)]" href="mailto:hello@example.com">
      Share feedback
    </a>
  </div>
</section>
```

### Task 5: Add footer and ensure accessibility polish

**Files:**
- Modify: `site/index.html`

**Step 1: Add minimal footer**

```html
<footer class="border-t border-[color:var(--divider)] pt-6 text-xs text-[color:var(--muted-ink)]">
  <p>macOS-only. Local files. Designed to be emptied.</p>
</footer>
```

**Step 2: Manual verification**

Run: Open `site/index.html` in a browser.
Expected: Layout fits mobile and desktop widths, video embeds, focus rings visible, and CTAs link to GitHub releases.
