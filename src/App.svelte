<script lang="ts">
  import "./styles.css";
  import CommitList from "./lib/components/CommitList.svelte";
  import EditorPanel from "./lib/components/EditorPanel.svelte";
  import {
    openRepository,
    getCommits,
    getCommitDetail,
    updateCommit,
    type CommitSummary,
    type CommitDetail,
    type UpdateCommitParams,
    type RepoInfo,
  } from "./lib/api/commands";
  import { open } from "@tauri-apps/plugin-dialog";

  const PAGE_SIZE = 100;

  let repoPath = $state("");
  let repoInfo = $state<RepoInfo | null>(null);
  let commits = $state<CommitSummary[]>([]);
  let selectedOid = $state("");
  let selectedCommit = $state<CommitDetail | null>(null);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state("");
  let pathInput = $state("");
  let lastSaveResult = $state("");

  async function handleOpenRepo() {
    const path = pathInput.trim();
    if (!path) return;

    error = "";
    loading = true;
    try {
      const info = await openRepository(path);
      repoInfo = info;
      repoPath = path;
      commits = [];
      selectedOid = "";
      selectedCommit = null;
      loading = false;
      await loadMoreCommits();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function loadMoreCommits() {
    if (loading || !repoPath) return;
    if (repoInfo && commits.length >= repoInfo.commit_count) return;

    loading = true;
    try {
      const newCommits = await getCommits(repoPath, commits.length, PAGE_SIZE);
      commits = [...commits, ...newCommits];
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleSelectCommit(oid: string) {
    error = "";
    try {
      selectedCommit = await getCommitDetail(repoPath, oid);
    } catch (e) {
      error = String(e);
    }
  }

  async function handleSave(params: UpdateCommitParams) {
    saving = true;
    error = "";
    lastSaveResult = "";
    try {
      const result = await updateCommit(params);
      lastSaveResult = `Rewrote ${result.commits_rewritten} commit(s). New hash: ${result.new_oid.slice(0, 7)}`;

      // Reload the commit list and select the new commit
      commits = [];
      const info = await openRepository(repoPath);
      repoInfo = info;
      await loadMoreCommits();

      selectedOid = result.new_oid;
      selectedCommit = await getCommitDetail(repoPath, result.new_oid);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function handleBrowse() {
    const selected = await open({ directory: true, multiple: false, title: "Select Git Repository" });
    if (selected) {
      pathInput = selected as string;
      handleOpenRepo();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      handleOpenRepo();
    }
  }
</script>

<div class="app">
  {#if !repoPath}
    <div class="welcome">
      <h1>Git History Editor</h1>
      <p>Open a repository to start editing its commit history.</p>
      <div class="repo-input-group">
        <input
          type="text"
          bind:value={pathInput}
          placeholder="/path/to/repository"
          onkeydown={handleKeydown}
          class="repo-input"
        />
        <button class="btn btn-secondary" onclick={handleBrowse} disabled={loading}>
          Browse
        </button>
        <button class="btn btn-primary" onclick={handleOpenRepo} disabled={loading}>
          {loading ? "Opening..." : "Open"}
        </button>
      </div>
      {#if error}
        <p class="error">{error}</p>
      {/if}
    </div>
  {:else}
    <div class="toolbar">
      <div class="toolbar-left">
        <button class="btn btn-secondary btn-sm" onclick={() => { repoPath = ""; repoInfo = null; commits = []; selectedCommit = null; error = ""; lastSaveResult = ""; }}>
          &larr; Back
        </button>
        <span class="repo-name">{repoInfo?.path}</span>
        <span class="branch-badge">{repoInfo?.branch}</span>
        <span class="commit-count">{repoInfo?.commit_count} commits</span>
      </div>
      {#if error}
        <span class="toolbar-error">{error}</span>
      {/if}
      {#if lastSaveResult}
        <span class="toolbar-success">{lastSaveResult}</span>
      {/if}
    </div>
    <div class="main-content">
      <div class="left-panel">
        <CommitList
          {commits}
          bind:selectedOid
          onselect={handleSelectCommit}
          onloadmore={loadMoreCommits}
          {loading}
          totalCount={repoInfo?.commit_count ?? 0}
        />
      </div>
      <div class="right-panel">
        <EditorPanel
          commit={selectedCommit}
          {repoPath}
          onsave={handleSave}
          {saving}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .app {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 16px;
  }

  .welcome h1 {
    font-size: 24px;
    font-weight: 600;
  }

  .welcome p {
    color: var(--text-secondary);
  }

  .repo-input-group {
    display: flex;
    gap: 8px;
    width: 100%;
    max-width: 500px;
  }

  .repo-input {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    outline: none;
  }

  .repo-input:focus {
    border-color: var(--accent);
  }

  .error {
    color: var(--danger);
    font-size: 12px;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    gap: 12px;
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .repo-name {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .branch-badge {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 10px;
    background: var(--accent);
    color: var(--bg-primary);
    font-weight: 600;
    white-space: nowrap;
  }

  .commit-count {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .toolbar-error {
    color: var(--danger);
    font-size: 12px;
    white-space: nowrap;
  }

  .toolbar-success {
    color: var(--success);
    font-size: 12px;
    white-space: nowrap;
  }

  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .left-panel {
    width: 55%;
    min-width: 400px;
    border-right: 1px solid var(--border);
    overflow: hidden;
  }

  .right-panel {
    flex: 1;
    overflow: hidden;
  }

  .btn {
    padding: 6px 16px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.15s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }

  .btn-secondary {
    background: transparent;
    color: var(--text-secondary);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .btn-sm {
    padding: 3px 10px;
    font-size: 12px;
  }
</style>
