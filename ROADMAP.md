# Git History Editor — Roadmap to v1.0

## Phase 1 — Bug Fixes (must-do before any release)

- [x] ~~Fix timezone offset parsing bug~~ — Verified correct: `parseTimestamp` subtraction is the proper inverse of `formatTimestamp` addition. Roundtrip tested with positive and negative offsets.
- [x] Set a proper Content Security Policy in `tauri.conf.json` — Restrictive CSP now set (`default-src 'self'`, `style-src 'self' 'unsafe-inline'`, etc.)
- [x] Gate devtools to debug builds only — Moved to a Cargo feature flag; enabled via `npm run tauri:dev` only
- [x] Handle empty repos gracefully — `open_repository` and `get_commits` now return 0 commits instead of crashing; improved detached HEAD error message

## Phase 2 — Safety & Polish

- [x] Add undo/restore — Backup ref saved before each rewrite (`refs/git-history-editor/pre-rewrite/<branch>`), "Undo Last Rewrite" button in toolbar, `restore_backup` command resets branch and cleans up ref
- [ ] Create a proper app icon (512x512+, currently 32x32 placeholder)
- [x] Add progress indication for large history rewrites — Emits `rewrite-progress` events every 100 commits, progress bar shown in toolbar during rewrite
- [x] Improve error messages — Author/committer signature errors identify the field, permission errors on ref update, "not a git repo" detection, clear detached HEAD message

## Phase 3 — Testing & Distribution

- [x] Create `scripts/build.sh` — Builds universal binary, codesigns with Developer ID, verifies signature
- [x] Create `scripts/notarize.sh` — Submits `.app` or `.dmg` to Apple notarization, waits, staples ticket. Supports both API key and Apple ID auth.
- [x] Create `scripts/release.sh` — Full pipeline: build → sign → notarize app → create DMG → sign DMG → notarize DMG
- [x] Add tests — 11 Rust tests covering: open repo (with commits, empty, non-git), commit pagination, commit detail, rewrite (message, author, descendant preservation), backup/restore cycle
- [x] Add `tauri-plugin-updater` for auto-updates — "Check for updates" on welcome screen, downloads and relaunches. Endpoint placeholder for GitHub Releases. Network client entitlement added. CSP updated for GitHub connections.
- [x] Write privacy policy — `docs/privacy-policy.html`, states no data collection, explains local storage and update check

## Phase 4 — Nice to Have

- [x] Keyboard shortcuts — `⌘S` save, `Escape` discard/close dialog, `⌘O` browse for repo, `⌘Z` undo last rewrite (outside text inputs). Shortcut hints shown in editor footer.
- [x] Visual indicators for modified fields — Inputs with changed values get a yellow left border (`var(--warning)`). Co-author rows also highlighted when modified.
- [x] Search/filter commits by message or author — Search bar above commit list filters by message, author name/email, or hash prefix. Shows match count. `Escape` clears filter.
- [x] Co-authored-by trailer editing — Parses `Co-authored-by:` trailers from commit messages into editable name/email rows. Add/remove co-authors with dedicated UI. Trailers reconstructed on save.
