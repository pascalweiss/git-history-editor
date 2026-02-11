<script lang="ts">
  import type { CommitSummary } from "../api/commands";
  import CommitRow from "./CommitRow.svelte";

  let {
    commits,
    selectedOid = $bindable(""),
    onselect,
    onloadmore,
    loading = false,
    totalCount = 0,
  }: {
    commits: CommitSummary[];
    selectedOid?: string;
    onselect: (oid: string) => void;
    onloadmore: () => void;
    loading?: boolean;
    totalCount?: number;
  } = $props();

  let listContainer: HTMLElement;
  let searchQuery = $state("");

  let filteredCommits = $derived(
    searchQuery.trim()
      ? commits.filter((c) => {
          const q = searchQuery.toLowerCase();
          return (
            c.short_message.toLowerCase().includes(q) ||
            c.author_name.toLowerCase().includes(q) ||
            c.author_email.toLowerCase().includes(q) ||
            c.oid.toLowerCase().startsWith(q)
          );
        })
      : commits
  );

  function handleScroll() {
    if (!listContainer || loading) return;
    const { scrollTop, scrollHeight, clientHeight } = listContainer;
    if (scrollHeight - scrollTop - clientHeight < 200) {
      onloadmore();
    }
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      searchQuery = "";
    }
  }
</script>

<div class="commit-list">
  <div class="search-bar">
    <input
      type="text"
      bind:value={searchQuery}
      onkeydown={handleSearchKeydown}
      placeholder="Filter by message, author, or hash..."
      class="search-input"
    />
    {#if searchQuery}
      <span class="search-count">{filteredCommits.length} of {commits.length}</span>
    {/if}
  </div>
  <div class="list-header">
    <span class="header-oid">Hash</span>
    <span class="header-message">Message</span>
    <span class="header-author">Author</span>
    <span class="header-date">Date</span>
  </div>
  <div class="list-body" bind:this={listContainer} onscroll={handleScroll}>
    {#each filteredCommits as commit (commit.oid)}
      <CommitRow
        {commit}
        selected={commit.oid === selectedOid}
        onclick={() => {
          selectedOid = commit.oid;
          onselect(commit.oid);
        }}
      />
    {/each}
    {#if loading}
      <div class="loading">Loading more commits...</div>
    {/if}
    {#if !loading && !searchQuery && commits.length > 0 && commits.length >= totalCount}
      <div class="end-marker">End of history ({totalCount} commits)</div>
    {/if}
    {#if !loading && filteredCommits.length === 0 && searchQuery}
      <div class="empty">No matching commits</div>
    {/if}
    {#if !loading && commits.length === 0 && !searchQuery}
      <div class="empty">No commits found</div>
    {/if}
  </div>
</div>

<style>
  .commit-list {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
  }

  .search-input {
    flex: 1;
    padding: 5px 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
  }

  .search-input:focus {
    border-color: var(--accent);
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .search-count {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .list-header {
    display: grid;
    grid-template-columns: 70px 1fr 150px 100px;
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .header-date {
    text-align: right;
  }

  .list-body {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .loading, .end-marker, .empty {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
