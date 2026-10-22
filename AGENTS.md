# Agent instructions

Follow project documentation and preserve existing content.

## Beads

This repository uses Beads with an embedded Dolt database. Configure Git hooks with `git config core.hooksPath .beads/hooks`. At session start, run `git pull`, `bd dolt pull`, `bd status`, and `bd list --label=focus` (read notes on focus beads). Git hooks also pull Beads after merges and branch checkouts. At session end, commit work and run `git push`; the repo-local pre-push hook pushes Beads Dolt data too. If that hook reports a sync failure, run `bd dolt push` manually. JSONL export is for interchange and viewers; Dolt remotes provide cross-machine sync.

## Changelog

At session end, only log new features or significant reader-facing changes
(behaviour, fixes to materially broken functionality, or public content).
Skip polish, tidying, and small fixes as standalone entries; include them only
as supporting detail in a larger feature or release announcement that qualifies
on its own. Planning/research/design must itself be a significant public
deliverable. When unsure, skip.

Use `changelog/YYYY-MM-DD-slug.md` with `date`/`title`/`promote` frontmatter:
a plain-text title, one or two reader-facing sentences, a live link where
available, and consider a screenshot for visual work. Substantial announcements
can use paragraphs or bullets. Omit internal implementation detail. For the
first entry or when the format is unclear, fetch and follow
https://raw.githubusercontent.com/life-itself/changelog/main/CONVENTION.md
