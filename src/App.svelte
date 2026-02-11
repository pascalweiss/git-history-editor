<script lang="ts">
  import "./styles.css";
  import CommitList from "./lib/components/CommitList.svelte";
  import EditorPanel from "./lib/components/EditorPanel.svelte";
  import {
    openRepository,
    getCommits,
    getCommitDetail,
    updateCommit,
    checkBackup,
    restoreBackup,
    type CommitSummary,
    type CommitDetail,
    type UpdateCommitParams,
    type RepoInfo,
    type BackupInfo,
  } from "./lib/api/commands";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";

  const PAGE_SIZE = 100;
  const MAX_RECENT_REPOS = 10;

  let repoPath = $state("");
  let repoInfo = $state<RepoInfo | null>(null);
  let recentRepos = $state<string[]>([]);
  let store: Store | null = null;
  let commits = $state<CommitSummary[]>([]);
  let selectedOid = $state("");
  let selectedCommit = $state<CommitDetail | null>(null);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state("");
  let pathInput = $state("");
  let lastSaveResult = $state("");
  let backup = $state<BackupInfo | null>(null);
  let restoring = $state(false);
  let rewriteProgress = $state<{ current: number; total: number } | null>(null);
  let unlistenProgress: UnlistenFn | null = null;
  let updateStatus = $state<"idle" | "checking" | "available" | "downloading" | "ready" | "error">("idle");
  let updateVersion = $state("");
  let updateError = $state("");

  async function setupProgressListener() {
    unlistenProgress = await listen<{ current: number; total: number }>("rewrite-progress", (event) => {
      rewriteProgress = event.payload;
    });
  }

  setupProgressListener();

  async function checkForUpdates() {
    updateStatus = "checking";
    updateError = "";
    try {
      const update = await check();
      if (update) {
        updateVersion = update.version;
        updateStatus = "available";
      } else {
        updateStatus = "idle";
      }
    } catch (e) {
      updateError = String(e);
      updateStatus = "error";
    }
  }

  async function downloadAndInstallUpdate() {
    updateStatus = "downloading";
    try {
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        updateStatus = "ready";
        await relaunch();
      }
    } catch (e) {
      updateError = String(e);
      updateStatus = "error";
    }
  }

  async function initStore() {
    store = await load("recent-repos.json", { autoSave: true });
    const saved = await store.get<string[]>("recentRepos");
    if (saved) {
      recentRepos = saved;
    }
  }

  async function addRecentRepo(path: string) {
    recentRepos = [path, ...recentRepos.filter((r) => r !== path)].slice(0, MAX_RECENT_REPOS);
    await store?.set("recentRepos", recentRepos);
  }

  async function removeRecentRepo(path: string) {
    recentRepos = recentRepos.filter((r) => r !== path);
    await store?.set("recentRepos", recentRepos);
  }

  initStore();

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
      await addRecentRepo(path);
      await loadMoreCommits();
      backup = await checkBackup(path);
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
    rewriteProgress = null;
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
      backup = await checkBackup(repoPath);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
      rewriteProgress = null;
    }
  }

  async function handleBrowse() {
    const selected = await open({ directory: true, multiple: false, recursive: true, title: "Select Git Repository" });
    if (selected) {
      pathInput = selected as string;
      handleOpenRepo();
    }
  }

  async function handleRestore() {
    if (!repoPath || restoring) return;
    restoring = true;
    error = "";
    lastSaveResult = "";
    try {
      await restoreBackup(repoPath);
      lastSaveResult = "Restored to pre-rewrite state";

      commits = [];
      const info = await openRepository(repoPath);
      repoInfo = info;
      selectedOid = "";
      selectedCommit = null;
      await loadMoreCommits();
      backup = await checkBackup(repoPath);
    } catch (e) {
      error = String(e);
    } finally {
      restoring = false;
    }
  }

  function handleInputKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      handleOpenRepo();
    }
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
    if (mod && e.key === "o") {
      e.preventDefault();
      if (!repoPath) {
        handleBrowse();
      }
    } else if (mod && e.key === "z" && !e.shiftKey) {
      // Only handle Cmd+Z for undo rewrite when in repo view and backup exists
      // Don't interfere with normal text undo in inputs
      const target = e.target as HTMLElement;
      if (target?.tagName === "INPUT" || target?.tagName === "TEXTAREA") return;
      if (repoPath && backup?.exists && !restoring && !saving) {
        e.preventDefault();
        handleRestore();
      }
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

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
          onkeydown={handleInputKeydown}
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
      {#if recentRepos.length > 0}
        <div class="recent-repos">
          <h3>Recent Repositories</h3>
          <ul class="recent-list">
            {#each recentRepos as repo}
              <li class="recent-item">
                <button
                  class="recent-link"
                  onclick={() => { pathInput = repo; handleOpenRepo(); }}
                  disabled={loading}
                >
                  {repo}
                </button>
                <button
                  class="recent-remove"
                  onclick={() => removeRecentRepo(repo)}
                  title="Remove from recent"
                >&times;</button>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
      <div class="update-section">
        {#if updateStatus === "idle"}
          <button class="update-link" onclick={checkForUpdates}>Check for updates</button>
        {:else if updateStatus === "checking"}
          <span class="update-text">Checking for updates...</span>
        {:else if updateStatus === "available"}
          <span class="update-text">Version {updateVersion} available.</span>
          <button class="update-link" onclick={downloadAndInstallUpdate}>Download and install</button>
        {:else if updateStatus === "downloading"}
          <span class="update-text">Downloading update...</span>
        {:else if updateStatus === "ready"}
          <span class="update-text">Restarting...</span>
        {:else if updateStatus === "error"}
          <span class="update-text update-error">{updateError}</span>
          <button class="update-link" onclick={checkForUpdates}>Retry</button>
        {/if}
      </div>
    </div>
  {:else}
    <div class="toolbar">
      <div class="toolbar-left">
        <button class="btn btn-secondary btn-sm" onclick={() => { repoPath = ""; repoInfo = null; commits = []; selectedCommit = null; error = ""; lastSaveResult = ""; backup = null; }}>
          &larr; Back
        </button>
        <span class="repo-name">{repoInfo?.path}</span>
        <span class="branch-badge">{repoInfo?.branch}</span>
        <span class="commit-count">{repoInfo?.commit_count} commits</span>
      </div>
      <div class="toolbar-right">
        {#if backup?.exists}
          <button class="btn btn-warning btn-sm" onclick={handleRestore} disabled={restoring || saving}>
            {restoring ? "Restoring..." : "Undo Last Rewrite"}
          </button>
        {/if}
        {#if saving && rewriteProgress}
          <div class="progress-container">
            <div class="progress-bar" style="width: {Math.round((rewriteProgress.current / rewriteProgress.total) * 100)}%"></div>
            <span class="progress-text">Rewriting... {rewriteProgress.current}/{rewriteProgress.total}</span>
          </div>
        {/if}
        {#if error}
          <span class="toolbar-error">{error}</span>
        {/if}
        {#if lastSaveResult}
          <span class="toolbar-success">{lastSaveResult}</span>
        {/if}
      </div>
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

  .recent-repos {
    width: 100%;
    max-width: 500px;
    margin-top: 16px;
  }

  .recent-repos h3 {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 8px;
  }

  .recent-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .recent-item {
    display: flex;
    align-items: center;
    gap: 4px;
    border-radius: var(--radius);
  }

  .recent-item:hover {
    background: var(--bg-hover);
  }

  .recent-link {
    flex: 1;
    background: none;
    border: none;
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 6px 8px;
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .recent-link:hover:not(:disabled) {
    text-decoration: underline;
  }

  .recent-link:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .recent-remove {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px 8px;
    font-size: 14px;
    line-height: 1;
    border-radius: var(--radius);
  }

  .recent-remove:hover {
    color: var(--danger);
    background: var(--bg-secondary);
  }

  .update-section {
    margin-top: 24px;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .update-link {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 12px;
    padding: 0;
    text-decoration: underline;
  }

  .update-link:hover {
    color: var(--accent);
  }

  .update-text {
    color: var(--text-muted);
  }

  .update-error {
    color: var(--danger);
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

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }

  .btn-warning {
    background: var(--warning);
    color: var(--bg-primary);
    border-color: var(--warning);
  }

  .btn-warning:hover:not(:disabled) {
    opacity: 0.9;
  }

  .progress-container {
    position: relative;
    width: 180px;
    height: 20px;
    background: var(--bg-secondary);
    border-radius: var(--radius);
    overflow: hidden;
    flex-shrink: 0;
  }

  .progress-bar {
    height: 100%;
    background: var(--accent);
    transition: width 0.1s;
  }

  .progress-text {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: var(--text-primary);
    font-weight: 500;
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
