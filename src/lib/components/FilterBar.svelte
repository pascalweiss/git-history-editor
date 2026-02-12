<script lang="ts">
  import type { CommitFilters } from "../api/commands";

  let {
    filters = $bindable({} as CommitFilters),
    onfilterchange,
    authors = [],
  }: {
    filters: CommitFilters;
    onfilterchange: () => void;
    authors?: string[];
  } = $props();

  let showAdvanced = $state(false);
  let dateStartInput = $state("");
  let dateEndInput = $state("");

  // Update date filters when date inputs change
  $effect(() => {
    if (dateStartInput) {
      const date = new Date(dateStartInput);
      filters.date_start = Math.floor(date.getTime() / 1000);
    } else {
      filters.date_start = undefined;
    }
  });

  $effect(() => {
    if (dateEndInput) {
      const date = new Date(dateEndInput);
      // Set to end of day
      date.setHours(23, 59, 59, 999);
      filters.date_end = Math.floor(date.getTime() / 1000);
    } else {
      filters.date_end = undefined;
    }
  });

  function formatDate(timestamp?: number): string {
    if (!timestamp) return "";
    return new Date(timestamp * 1000).toISOString().split('T')[0];
  }

  function clearFilter(key: keyof CommitFilters) {
    filters[key] = undefined;
    if (key === "date_start") dateStartInput = "";
    if (key === "date_end") dateEndInput = "";
    onfilterchange();
  }

  function clearAllFilters() {
    filters.author_name = undefined;
    filters.author_email = undefined;
    filters.message_pattern = undefined;
    filters.date_start = undefined;
    filters.date_end = undefined;
    filters.file_path = undefined;
    dateStartInput = "";
    dateEndInput = "";
    onfilterchange();
  }

  function hasActiveFilters(): boolean {
    return !!(
      filters.author_name ||
      filters.author_email ||
      filters.message_pattern ||
      filters.date_start ||
      filters.date_end ||
      filters.file_path
    );
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      onfilterchange();
    } else if (e.key === "Escape") {
      clearAllFilters();
    }
  }
</script>

<div class="filter-bar">
  <div class="filter-row">
    <input
      type="text"
      bind:value={filters.message_pattern}
      onkeydown={handleKeydown}
      placeholder="Filter by message..."
      class="filter-input flex-2"
    />
    <input
      type="text"
      bind:value={filters.author_name}
      onkeydown={handleKeydown}
      placeholder="Author name..."
      class="filter-input flex-1"
    />
    <button
      class="toggle-btn"
      onclick={() => showAdvanced = !showAdvanced}
      title={showAdvanced ? "Hide advanced filters" : "Show advanced filters"}
    >
      {showAdvanced ? "−" : "+"}
    </button>
    <button class="apply-btn" onclick={onfilterchange}>Apply</button>
    {#if hasActiveFilters()}
      <button class="clear-btn" onclick={clearAllFilters} title="Clear all filters">×</button>
    {/if}
  </div>

  {#if showAdvanced}
    <div class="filter-row advanced">
      <input
        type="text"
        bind:value={filters.author_email}
        onkeydown={handleKeydown}
        placeholder="Author email..."
        class="filter-input flex-1"
      />
      <input
        type="date"
        bind:value={dateStartInput}
        onchange={onfilterchange}
        placeholder="Start date"
        class="filter-input flex-1"
        title="Date from"
      />
      <input
        type="date"
        bind:value={dateEndInput}
        onchange={onfilterchange}
        placeholder="End date"
        class="filter-input flex-1"
        title="Date to"
      />
      <input
        type="text"
        bind:value={filters.file_path}
        onkeydown={handleKeydown}
        placeholder="File path (e.g., src/*.rs)..."
        class="filter-input flex-2"
      />
    </div>
  {/if}

  {#if hasActiveFilters()}
    <div class="active-filters">
      {#if filters.message_pattern}
        <span class="filter-chip">
          Message: {filters.message_pattern}
          <button onclick={() => clearFilter("message_pattern")}>×</button>
        </span>
      {/if}
      {#if filters.author_name}
        <span class="filter-chip">
          Author: {filters.author_name}
          <button onclick={() => clearFilter("author_name")}>×</button>
        </span>
      {/if}
      {#if filters.author_email}
        <span class="filter-chip">
          Email: {filters.author_email}
          <button onclick={() => clearFilter("author_email")}>×</button>
        </span>
      {/if}
      {#if filters.date_start}
        <span class="filter-chip">
          From: {formatDate(filters.date_start)}
          <button onclick={() => clearFilter("date_start")}>×</button>
        </span>
      {/if}
      {#if filters.date_end}
        <span class="filter-chip">
          To: {formatDate(filters.date_end)}
          <button onclick={() => clearFilter("date_end")}>×</button>
        </span>
      {/if}
      {#if filters.file_path}
        <span class="filter-chip">
          File: {filters.file_path}
          <button onclick={() => clearFilter("file_path")}>×</button>
        </span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .filter-bar {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
  }

  .filter-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .filter-row.advanced {
    padding-top: 4px;
  }

  .filter-input {
    padding: 5px 10px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
    min-width: 0;
  }

  .filter-input:focus {
    border-color: var(--accent);
  }

  .filter-input::placeholder {
    color: var(--text-muted);
  }

  .filter-input[type="date"] {
    font-family: var(--font-mono);
  }

  .flex-1 {
    flex: 1;
  }

  .flex-2 {
    flex: 2;
  }

  .toggle-btn {
    padding: 5px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-primary);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .toggle-btn:hover {
    background: var(--bg-hover);
    border-color: var(--accent);
  }

  .apply-btn {
    padding: 5px 16px;
    background: var(--accent);
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    color: var(--bg-primary);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .apply-btn:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }

  .clear-btn {
    padding: 5px 10px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text-muted);
    font-size: 16px;
    cursor: pointer;
    transition: all 0.15s;
    line-height: 1;
  }

  .clear-btn:hover {
    background: var(--danger);
    color: white;
    border-color: var(--danger);
  }

  .active-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding-top: 4px;
  }

  .filter-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    background: var(--accent);
    color: var(--bg-primary);
    border-radius: 12px;
    font-size: 11px;
    font-weight: 500;
  }

  .filter-chip button {
    background: none;
    border: none;
    color: var(--bg-primary);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 0;
    margin: 0;
    opacity: 0.8;
  }

  .filter-chip button:hover {
    opacity: 1;
  }
</style>
