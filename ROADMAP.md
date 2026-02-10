# Git History Editor — Roadmap to v1.0

## Phase 1 — Bug Fixes (must-do before any release)

- [x] ~~Fix timezone offset parsing bug~~ — Verified correct: `parseTimestamp` subtraction is the proper inverse of `formatTimestamp` addition. Roundtrip tested with positive and negative offsets.
- [x] Set a proper Content Security Policy in `tauri.conf.json` — Restrictive CSP now set (`default-src 'self'`, `style-src 'self' 'unsafe-inline'`, etc.)
- [x] Gate devtools to debug builds only — Moved to a Cargo feature flag; enabled via `npm run tauri:dev` only
- [x] Handle empty repos gracefully — `open_repository` and `get_commits` now return 0 commits instead of crashing; improved detached HEAD error message

## Phase 2 — Safety & Polish

- [ ] Add undo/restore: save backup refs before rewrite so users can recover
- [ ] Create a proper app icon (512x512+, currently 32x32 placeholder)
- [ ] Add progress indication for large history rewrites
- [ ] Improve error messages (identify which field failed, permission errors, detached HEAD explanation)

## Phase 3 — Testing & Distribution

- [ ] Add tests for the rewrite algorithm and timezone logic
- [ ] Create a `scripts/build.sh` script that produces a signed `.app` bundle (universal binary via `tauri build --target universal-apple-darwin`, codesigned with Developer ID)
- [ ] Create a `scripts/notarize.sh` script that submits the `.app` to Apple's notarization service via `xcrun notarytool` and staples the result
- [ ] Create a `scripts/release.sh` script that runs build + notarize end-to-end and produces a ready-to-distribute `.dmg`
- [ ] Add tests for the rewrite algorithm and timezone logic
- [ ] Add `tauri-plugin-updater` for auto-updates
- [ ] Write a privacy policy page (required even for notarized distribution)

## Phase 4 — Nice to Have

- [ ] Keyboard shortcuts (Cmd+S to save, Escape to cancel, etc.)
- [ ] Visual indicators for modified fields (highlight what changed before saving)
- [ ] Search/filter commits by message or author
- [ ] Co-authored-by trailer editing (mentioned in README but not yet implemented)
