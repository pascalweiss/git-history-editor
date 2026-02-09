# Git History Editor - Project Plan

## 1. Project Overview

A desktop application for visually editing git commit history. The UI features a split-pane layout: a scrollable commit list on the left (displaying info similar to `git log --oneline`), and a detail/editor panel on the right where the user can modify commit metadata (author, committer, message, dates, co-authors/trailers).

**Target platforms:** macOS, Linux (desktop only)

---

## 2. Technology Decision

### 2.1 Framework: Tauri 2 + Svelte 5

**Why Tauri over the alternatives:**

| Criterion | Electron | **Tauri 2** | Flutter | Python+Qt | Wails |
|---|---|---|---|---|---|
| Git lib quality | Good (CLI wrapper) | **Excellent (git2-rs)** | Poor | Excellent | Poor |
| Bundle size | 80-120 MB | **2-10 MB** | 20-40 MB | 50-80 MB | 5-15 MB |
| Memory usage | 200-300 MB | **30-40 MB** | 50-100 MB | 40-80 MB | 30-50 MB |
| Startup time | 1-2s | **<0.5s** | ~1s | ~1s | <0.5s |
| Cross-platform | Excellent | **Excellent** | Good | Excellent | Good |
| Ecosystem maturity | Excellent | **Good (2.0 stable)** | Good | Excellent | Fair |

**Rationale:**

- **git2-rs** is the Rust binding for libgit2, maintained under the `rust-lang` GitHub org. It provides low-level access to create, amend, and rewrite commits with full control over author, committer, message, tree, and parent references. This is the most robust programmatic git library available for this use case.
- Tauri 2.0 is stable (released late 2024), has 70k+ GitHub stars, and uses the OS-native WebView -- resulting in tiny bundles and low memory.
- For complex bulk operations, Tauri's shell plugin can invoke `git filter-repo` or raw `git` CLI commands as a fallback.
- Electron was the runner-up (proven by GitKraken/GitHub Desktop), but its 100+ MB bundle and 200+ MB RAM usage are unnecessary for a developer tool. Electron's `simple-git` library is just a CLI wrapper -- Tauri's git2-rs gives us *native* git object access, which is faster and more reliable.
- Flutter was rejected: Dart has no meaningful git library; all operations would require shelling out to the CLI, providing no advantage over Tauri while having a less mature desktop story.
- Python+Qt was rejected despite excellent git libraries (git-filter-repo, pygit2) because packaging/distribution of Python desktop apps is significantly harder and the UI feels less modern.

**Why Svelte 5 for the frontend:**

- Svelte compiles to vanilla JS (no virtual DOM), resulting in smaller bundles and better performance in a WebView context.
- Svelte 5's runes system provides reactive state management with minimal boilerplate.
- Simpler than React for a focused desktop app (no need for React's massive ecosystem).
- Excellent TypeScript support.

### 2.2 Git Operations: git2-rs (Rust, via libgit2)

**Capabilities we need and how git2-rs covers them:**

| Operation | git2-rs API | Notes |
|---|---|---|
| List commits | `Revwalk` + `Repository::find_commit()` | Topological walk, paginated |
| Read commit details | `Commit::message()`, `author()`, `committer()`, `tree()`, `parent_count()` | Full metadata access |
| Read diff for a commit | `Repository::diff_tree_to_tree()` | Compare commit tree vs parent tree |
| Amend HEAD commit | `Commit::amend()` | Changes author/committer/message, preserves parents |
| Rewrite commit in history | `Repository::commit()` with remapped parents | See Section 3.3 |
| Update refs after rewrite | `Repository::reference()` | Point branch to new tip |

**The core rewrite algorithm** (for editing a commit that is not HEAD):

1. Walk commits from root to tip using `Revwalk` with `TOPOLOGICAL | REVERSE` sorting.
2. Maintain an `old_oid -> new_oid` HashMap.
3. For each commit:
   - If it's the target commit, apply the user's edits (new author, message, etc.).
   - Remap all parent OIDs using the HashMap (if a parent was rewritten, use its new OID).
   - Create a new commit via `repo.commit()` with the (possibly modified) metadata and remapped parents.
   - Store the `old_oid -> new_oid` mapping.
4. Update the branch ref to point to the new tip commit.

This algorithm correctly handles the hash cascade (every descendant of a modified commit gets a new hash because its parent hash changed).

**Edge cases handled:**
- Merge commits: all parent OIDs are remapped (not just the first parent).
- Co-authored-by trailers: these are part of the commit message body, so editing them is just message string manipulation.
- GPG signatures: rewritten commits lose their signatures (unavoidable; the hash changes).

---

## 3. Architecture

```
+----------------------------------------------------------+
|                    Tauri Window                           |
|  +------------------------+  +--------------------------+|
|  |   Commit List (Svelte) |  |  Editor Panel (Svelte)   ||
|  |                        |  |                          ||
|  |  abc1234 Fix login bug |  |  Hash: abc1234def5678    ||
|  |  def5678 Add tests     |  |  Author: John Doe        ||
|  |  > 789abcd Refactor DB |  |  Email: john@example.com ||
|  |  ...                   |  |  Date: 2025-01-15 14:30  ||
|  |                        |  |  Message: [editable]     ||
|  |  (virtual scroll)      |  |  Co-authors: [editable]  ||
|  |                        |  |  Committer: [editable]   ||
|  +------------------------+  +--------------------------+|
+----------------------------------------------------------+
         |  Tauri IPC (invoke)     ^
         v                         |
+----------------------------------------------------------+
|                  Rust Backend                            |
|                                                          |
|  +------------------+  +-------------------------------+ |
|  | git2-rs          |  | History Rewriter              | |
|  | (libgit2 bindings)|  | - walk commits (revwalk)     | |
|  |                  |  | - remap parents               | |
|  | - read commits   |  | - create new commits          | |
|  | - read diffs     |  | - update refs                 | |
|  | - read refs      |  | - OID map tracking            | |
|  +------------------+  +-------------------------------+ |
|                                                          |
|  +--------------------------------------------------+   |
|  | Repository State                                  |   |
|  | - Opened repo path                                |   |
|  | - Cached commit list                              |   |
|  | - Pending edits queue                             |   |
|  +--------------------------------------------------+   |
+----------------------------------------------------------+
```

### 3.1 Tauri Commands (Rust -> Frontend IPC)

```rust
// Core commands exposed to the Svelte frontend:

#[tauri::command]
fn open_repository(path: String) -> Result<RepoInfo, String>;

#[tauri::command]
fn get_commits(offset: usize, limit: usize) -> Result<Vec<CommitSummary>, String>;

#[tauri::command]
fn get_commit_detail(oid: String) -> Result<CommitDetail, String>;

#[tauri::command]
fn update_commit(
    oid: String,
    new_author_name: Option<String>,
    new_author_email: Option<String>,
    new_author_date: Option<String>,
    new_committer_name: Option<String>,
    new_committer_email: Option<String>,
    new_committer_date: Option<String>,
    new_message: Option<String>,
) -> Result<RewriteResult, String>;
```

### 3.2 Data Models

```rust
struct CommitSummary {
    oid: String,           // abbreviated hash
    short_message: String, // first ~72 chars of message
    author_name: String,
    author_date: String,   // ISO 8601
}

struct CommitDetail {
    oid: String,             // full hash
    message: String,         // full message including trailers
    author_name: String,
    author_email: String,
    author_date: String,
    committer_name: String,
    committer_email: String,
    committer_date: String,
    parent_oids: Vec<String>,
    is_merge: bool,
}

struct RewriteResult {
    old_oid: String,
    new_oid: String,
    commits_rewritten: usize,  // number of descendants also rewritten
}
```

### 3.3 Frontend Components (Svelte 5)

```
src/
  lib/
    components/
      CommitList.svelte       # Virtual-scrolling list of commits
      CommitRow.svelte         # Single row: hash, message, author, date
      EditorPanel.svelte       # Right panel: form fields for editing
      MessageEditor.svelte     # Textarea for commit message + trailer parsing
      TrailerEditor.svelte     # Co-authored-by and other trailers
    stores/
      repo.svelte.ts           # Reactive state: commits, selected commit, repo path
    api/
      commands.ts              # Typed wrappers around Tauri invoke() calls
  App.svelte                   # Root: split-pane layout
  main.ts                      # Entry point
```

**Key UI behaviors:**
- **Virtual scrolling** for the commit list (essential for repos with thousands of commits). Use a library like `svelte-virtual-list` or implement a simple virtual scroller.
- **Split-pane** resizable layout. Use a CSS grid or a small library.
- **Confirmation dialog** before applying edits, showing a summary of what will change and how many commits will be rewritten.
- **Progress indicator** for rewrites that touch many commits (the Rust backend can emit progress events via Tauri's event system).

---

## 4. Implementation Plan

### Phase 1: Project Scaffolding

1. Initialize Tauri 2 project with Svelte 5 frontend.
2. Set up Rust dependencies: `git2`, `serde`, `serde_json`.
3. Set up frontend dependencies: Svelte 5, TypeScript.
4. Verify the app builds and opens a window on macOS.

### Phase 2: Read-Only Commit Viewer

5. Implement `open_repository` command -- open a git repo by path, return basic info (name, branch, commit count).
6. Implement `get_commits` command -- paginated commit list via `Revwalk`.
7. Build the `CommitList` component with virtual scrolling.
8. Implement `get_commit_detail` command -- full metadata for a selected commit.
9. Build the `EditorPanel` component (read-only at first) showing all commit fields.
10. Wire up selection: clicking a commit in the list populates the editor panel.

### Phase 3: History Editing

11. Implement the core history rewrite function in Rust:
    - Accept a target commit OID and a set of field overrides.
    - Walk from root to tip, remap parents, create new commits.
    - Update the branch ref.
    - Return the rewrite result (new OID, count of rewritten commits).
12. Make the editor panel fields editable (author name, email, date, committer fields, message).
13. Add a "Save" button that invokes `update_commit` via Tauri IPC.
14. Add a confirmation dialog before applying changes.
15. After a successful rewrite, refresh the commit list and re-select the (now new) commit.

### Phase 4: Co-Author / Trailer Editing

16. Parse `Co-authored-by:` and other standard trailers from the commit message.
17. Build a `TrailerEditor` component that shows trailers as structured fields (name + email).
18. Allow adding, removing, and editing trailers.
19. On save, reconstruct the message body with updated trailers.

### Phase 5: Polish & Safety

20. Add progress events for long rewrites (Tauri event system).
21. Add error handling and user-friendly error messages.
22. Add a "discard changes" / undo mechanism (store the old branch ref before rewrite, offer to restore it).
23. Add a repo picker (file dialog or recent repos list).
24. Test on Linux (ensure Tauri WebView works, git2-rs builds correctly).

---

## 5. Key Dependencies

### Rust (Cargo.toml)

| Crate | Purpose |
|---|---|
| `tauri` (v2) | Desktop app framework |
| `git2` | libgit2 bindings for all git operations |
| `serde` + `serde_json` | Serialization for IPC |
| `chrono` | Date/time parsing and formatting |

### Frontend (package.json)

| Package | Purpose |
|---|---|
| `@tauri-apps/api` | Tauri frontend bindings (invoke, events) |
| `svelte` (v5) | UI framework |
| `typescript` | Type safety |

### System Requirements

- **git**: Must be installed on the user's system (for potential fallback CLI operations).
- **libgit2**: Bundled by the `git2` crate (no system install needed).
- **WebView**: WebKitGTK on Linux, WebKit on macOS (Tauri requirement).

---

## 6. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Rewriting history on a shared/pushed branch | Show a warning if the branch has a remote tracking branch |
| Data loss from buggy rewrite | Always store the pre-rewrite ref (like `git rebase` stores `ORIG_HEAD`); offer restore |
| Large repos (100k+ commits) slow to load | Paginated/virtual-scrolled commit list; lazy-load details on selection |
| Merge commits complicate rewriting | The rewrite algorithm remaps all parents; test with merge-heavy repos |
| GPG-signed commits lose signatures | Warn the user that signatures will be stripped on rewritten commits |
| libgit2 edge cases | For complex scenarios, fall back to `git` CLI via Tauri shell plugin |

---

## 7. Future Enhancements (Out of Scope for MVP)

- Batch editing (apply changes to multiple commits at once)
- Squash/split commits
- Reorder commits (drag-and-drop in the list)
- Diff viewer for each commit
- Support for `git filter-repo` integration for bulk operations
- Windows support
