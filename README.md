# Git History Editor

A lightweight desktop application for visually editing Git commit history. Browse your commits in a clean split-pane interface, select any commit, and modify its metadata — author, committer, dates, message, and co-author trailers — with a single click.

Built with [Tauri 2](https://v2.tauri.app/), [Svelte 5](https://svelte.dev/), and [git2-rs](https://github.com/rust-lang/git2-rs) (libgit2 bindings for Rust).

## Features

- **Commit browser** — Scrollable list of all commits with hash, message, author, and relative date
- **Inline editor** — Select any commit to edit author name/email, committer name/email, dates, and the full commit message (including `Co-authored-by` trailers)
- **Safe history rewriting** — Correctly propagates hash changes through all descendant commits; confirmation dialog before any destructive operation
- **Native folder picker** — Browse to any local Git repository
- **Tiny footprint** — ~5 MB bundle, ~30 MB RAM (uses the OS-native WebView, not Chromium)

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.70
- Platform-specific Tauri dependencies:
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: see [Tauri prerequisites for Linux](https://v2.tauri.app/start/prerequisites/#linux)

## Getting Started

```bash
# Clone the repository
git clone https://github.com/pascalweiss/git-history-editor.git
cd git-history-editor

# Install frontend dependencies
npm install

# Run in development mode
npx tauri dev
```

## Building for Production

```bash
npx tauri build
```

The bundled app will be in `src-tauri/target/release/bundle/`.

## How It Works

When you edit a commit, the Rust backend:

1. Walks all commits from root to tip in topological order
2. When it reaches the target commit, applies your changes (new author, message, etc.)
3. Remaps parent references for every descendant commit (since parent hashes change)
4. Creates new commit objects with the updated metadata
5. Updates the branch ref to point to the new tip

This is equivalent to what `git filter-branch` or `git filter-repo` does under the hood, but implemented natively through libgit2 for speed and safety.

> **Warning**: Rewriting history changes commit hashes for the edited commit and all of its descendants. Only use this on branches that haven't been shared, or coordinate with your team before force-pushing.

## Tech Stack

| Layer | Technology |
|---|---|
| Framework | [Tauri 2](https://v2.tauri.app/) |
| Frontend | [Svelte 5](https://svelte.dev/) + TypeScript |
| Backend | Rust |
| Git operations | [git2](https://crates.io/crates/git2) (libgit2, vendored) |
| Build tool | [Vite 6](https://vite.dev/) |

## Project Structure

```
├── src/                        # Svelte frontend
│   ├── App.svelte              # Root layout (welcome screen + split pane)
│   └── lib/
│       ├── api/commands.ts     # Typed Tauri IPC wrappers
│       └── components/         # CommitList, CommitRow, EditorPanel
├── src-tauri/                  # Rust backend
│   └── src/
│       ├── lib.rs              # Tauri plugin registration
│       └── git_commands.rs     # Git operations (read, rewrite)
├── index.html
├── vite.config.ts
└── package.json
```

## License

MIT
