# Website

`site/` is the static website deployed by the existing Cloudflare Pages GitHub integration (project `inboxapp`) when `main` changes. The public site is <https://tryinbox.app/>.

## Changelog

Write reader-facing entries in the repository's `changelog/` folder, following the convention linked from `AGENTS.md`. Then run:

```sh
npm run build:site
npm run check:site
```

Commit the generated `site/changelog/` files together with the source entry. They are checked in so the existing static deployment needs no dashboard or build-setting changes. The website check in GitHub Actions rejects stale generated HTML or images. Images are copied from `changelog/images/`; Markdown links to those images remain relative.

The `promote` field concerns optional promotion to other channels. All project changelog entries appear on this project's website, as requested. Release Please independently maintains `CHANGELOG.md` and GitHub release notes; it does not replace the reader-facing snippets. Mark unreleased features explicitly until a packaged release is available.
