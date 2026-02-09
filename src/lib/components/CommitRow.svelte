<script lang="ts">
  import type { CommitSummary } from "../api/commands";

  let {
    commit,
    selected = false,
    onclick,
  }: {
    commit: CommitSummary;
    selected?: boolean;
    onclick: () => void;
  } = $props();

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "today";
    if (diffDays === 1) return "yesterday";
    if (diffDays < 30) return `${diffDays} days ago`;
    if (diffDays < 365) {
      const months = Math.floor(diffDays / 30);
      return `${months} month${months > 1 ? "s" : ""} ago`;
    }
    const years = Math.floor(diffDays / 365);
    return `${years} year${years > 1 ? "s" : ""} ago`;
  }
</script>

<button class="commit-row" class:selected onclick={onclick}>
  <span class="oid">{commit.oid.slice(0, 7)}</span>
  <span class="message">{commit.short_message}</span>
  <span class="author">{commit.author_name}</span>
  <span class="date">{formatDate(commit.author_date)}</span>
</button>

<style>
  .commit-row {
    display: grid;
    grid-template-columns: 70px 1fr 150px 100px;
    gap: 8px;
    align-items: center;
    padding: 6px 12px;
    border: none;
    background: none;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    width: 100%;
    border-bottom: 1px solid var(--border);
    transition: background 0.1s;
  }

  .commit-row:hover {
    background: var(--bg-hover);
  }

  .commit-row.selected {
    background: var(--bg-surface);
    border-left: 2px solid var(--accent);
    padding-left: 10px;
  }

  .oid {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--accent);
  }

  .message {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .author {
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
  }

  .date {
    color: var(--text-muted);
    font-size: 12px;
    text-align: right;
  }
</style>
