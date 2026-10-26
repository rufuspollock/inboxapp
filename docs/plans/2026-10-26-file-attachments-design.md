# Arbitrary file attachments

Approved by Rufus on 26 October 2026. Work lives in the Beads epic “Capture
arbitrary file attachments including PDFs” (`inboxapp-d1l`) and its children.

Capture accepts any regular file up to 20 MB through paste or Attach file.
Finder clipboard files are copied into the entry's existing owned asset folder;
images copied from other apps retain screenshot paste behavior. Ordinary text
remains text. Arbitrary clipboard object formats and directories are outside this
file contract. The picker permits multiple files; successful copies remain linked
if a later import fails, with an explicit error for the failed file.

Files retain bytes, a safe version of their filename and extension, and receive
collision suffixes. PNG/JPEG/WebP files retain image previews when valid. Other
files use ordinary relative Markdown links and named buttons that open the owned
copy through macOS. No arbitrary document content is embedded in the WebView.
Ownership and local regular-file checks apply to all reads and opens.

Existing storage ownership, draft recovery and trash behavior stay in force.
Removing a Markdown link retains the asset; discarding an unsaved draft removes
its assets; deleting an entry moves all its assets into its trash bundle. No
unlinked-file cleanup is added.

Implementation follows tests first: extend owned storage and verify byte identity,
collisions and path guards; add native file clipboard handling with typed
no-content errors; extend the current attachment controls and editor tests. Use
the existing arboard dependency API directly for Finder files, with supported
WebKit image paste events and native image fallback. Keep the current UI styling.
An opt-in Mac clipboard harness exercises real native formats with temporary
storage and restores the pasteboard after testing. Native app automation then
verifies real capture, save, restart and open before installing the tested build.

Rust tests cover PDFs, unknown and empty files, filename collisions, limits,
ownership, symlinks and trash/discard. DOM tests cover file links, images, multiple
files, import failure, selection and named Open actions. Run full suites, build,
native checks and site freshness; document the outcome, then merge, push and sync
Beads. The existing 0.2.4 installation is backed up before replacement. Release
automation continues to own published version numbers.
