<script lang="ts">
  import type { CommitSummary, CommitFilters } from "../api/commands";
  import CommitRow from "./CommitRow.svelte";
  import FilterBar from "./FilterBar.svelte";

  let {
    commits,
    selectedOid = $bindable(""),
    onselect,
    onloadmore,
    onfilterchange,
    loading = false,
    totalCount = 0,
    filters = $bindable({} as CommitFilters),
  }: {
    commits: CommitSummary[];
    selectedOid?: string;
    onselect: (oid: string) => void;
    onloadmore: () => void;
    onfilterchange: (filters: CommitFilters) => void;
    loading?: boolean;
    totalCount?: number;
    filters?: CommitFilters;
  } = $props();

  let listContainer: HTMLElement;

  // Extract unique authors for autocomplete
  let authors = $derived(
    Array.from(new Set(commits.map((c) => c.author_name))).sort()
  );

  function handleScroll() {
    if (!listContainer || loading) return;
    const { scrollTop, scrollHeight, clientHeight } = listContainer;
    if (scrollHeight - scrollTop - clientHeight < 200) {
      onloadmore();
    }
  }

  function handleFilterChange() {
    onfilterchange(filters);
  }
</script>

<div class="commit-list">
  <FilterBar
    bind:filters
    onfilterchange={handleFilterChange}
    {authors}
  />
  <div class="list-header">
    <span class="header-oid">Hash</span>
    <span class="header-message">Message</span>
    <span class="header-author">Author</span>
    <span class="header-date">Date</span>
  </div>
  <div class="list-body" bind:this={listContainer} onscroll={handleScroll}>
    {#each commits as commit (commit.oid)}
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
    {#if !loading && commits.length === 0}
      <div class="empty">No matching commits found</div>
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

  .loading, .empty {
    padding: 16px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
