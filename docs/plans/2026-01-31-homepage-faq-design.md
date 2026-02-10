# Homepage FAQ (Why not just use X?) Design

## Goal
Add a simple FAQ section at the bottom of the homepage that answers the common question "Why not just use X?" with concrete examples and a concise, generic response aligned with Inbox's positioning.

## Placement
Insert a new section just above the footer in `site/index.html` so it reads as a closing clarification after the download CTA.

## Content
- Section title: "Why not just use X?"
- Single question line that names 2-3 examples: "Obsidian, Notion, or Todoist"
- Short answer (2-4 sentences) emphasizing:
  - Inbox is a deliberately dumb, frictionless capture layer
  - Other tools are excellent for organization but invite context switching/backlog browsing
  - Inbox keeps a blank sheet and outputs plain Markdown, so users can move items into their main system later

## Styling
Match existing layout patterns:
- Use the same grid layout as the "Why Inbox" section (`sm:grid-cols-2`)
- Keep copy in `text-sm` with muted ink and a slightly stronger question line
- Reuse existing border and spacing tokens; no new visual components
