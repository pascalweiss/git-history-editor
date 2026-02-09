<script lang="ts">
  import type { CommitDetail, UpdateCommitParams } from "../api/commands";

  let {
    commit,
    repoPath,
    onsave,
    saving = false,
  }: {
    commit: CommitDetail | null;
    repoPath: string;
    onsave: (params: UpdateCommitParams) => void;
    saving?: boolean;
  } = $props();

  let authorName = $state("");
  let authorEmail = $state("");
  let authorDateStr = $state("");
  let committerName = $state("");
  let committerEmail = $state("");
  let committerDateStr = $state("");
  let message = $state("");

  let showConfirm = $state(false);

  // Track the original values to detect changes
  let origAuthorName = "";
  let origAuthorEmail = "";
  let origAuthorDateStr = "";
  let origCommitterName = "";
  let origCommitterEmail = "";
  let origCommitterDateStr = "";
  let origMessage = "";

  $effect(() => {
    if (commit) {
      authorName = commit.author_name;
      authorEmail = commit.author_email;
      authorDateStr = formatTimestamp(commit.author_date, commit.author_offset);
      committerName = commit.committer_name;
      committerEmail = commit.committer_email;
      committerDateStr = formatTimestamp(commit.committer_date, commit.committer_offset);
      message = commit.message;

      origAuthorName = commit.author_name;
      origAuthorEmail = commit.author_email;
      origAuthorDateStr = authorDateStr;
      origCommitterName = commit.committer_name;
      origCommitterEmail = commit.committer_email;
      origCommitterDateStr = committerDateStr;
      origMessage = commit.message;
    }
  });

  function formatTimestamp(seconds: number, offsetMinutes: number): string {
    const date = new Date((seconds + offsetMinutes * 60) * 1000);
    // Format as a local datetime string for the input
    const year = date.getUTCFullYear();
    const month = String(date.getUTCMonth() + 1).padStart(2, "0");
    const day = String(date.getUTCDate()).padStart(2, "0");
    const hours = String(date.getUTCHours()).padStart(2, "0");
    const minutes = String(date.getUTCMinutes()).padStart(2, "0");
    const seconds_ = String(date.getUTCSeconds()).padStart(2, "0");
    return `${year}-${month}-${day}T${hours}:${minutes}:${seconds_}`;
  }

  function parseTimestamp(dateStr: string, originalOffset: number): { seconds: number; offset: number } {
    // Parse the datetime-local value back to a Unix timestamp
    const date = new Date(dateStr + "Z"); // Parse as UTC
    const seconds = Math.floor(date.getTime() / 1000) - originalOffset * 60;
    return { seconds, offset: originalOffset };
  }

  let hasChanges = $derived(
    authorName !== origAuthorName ||
    authorEmail !== origAuthorEmail ||
    authorDateStr !== origAuthorDateStr ||
    committerName !== origCommitterName ||
    committerEmail !== origCommitterEmail ||
    committerDateStr !== origCommitterDateStr ||
    message !== origMessage
  );

  function handleSave() {
    if (!commit || !hasChanges) return;
    showConfirm = true;
  }

  function confirmSave() {
    if (!commit) return;
    showConfirm = false;

    const params: UpdateCommitParams = {
      path: repoPath,
      oid: commit.oid,
    };

    if (authorName !== origAuthorName) params.newAuthorName = authorName;
    if (authorEmail !== origAuthorEmail) params.newAuthorEmail = authorEmail;
    if (authorDateStr !== origAuthorDateStr) {
      const parsed = parseTimestamp(authorDateStr, commit.author_offset);
      params.newAuthorDate = parsed.seconds;
      params.newAuthorOffset = parsed.offset;
    }
    if (committerName !== origCommitterName) params.newCommitterName = committerName;
    if (committerEmail !== origCommitterEmail) params.newCommitterEmail = committerEmail;
    if (committerDateStr !== origCommitterDateStr) {
      const parsed = parseTimestamp(committerDateStr, commit.committer_offset);
      params.newCommitterDate = parsed.seconds;
      params.newCommitterOffset = parsed.offset;
    }
    if (message !== origMessage) params.newMessage = message;

    onsave(params);
  }

  function handleDiscard() {
    if (commit) {
      authorName = origAuthorName;
      authorEmail = origAuthorEmail;
      authorDateStr = origAuthorDateStr;
      committerName = origCommitterName;
      committerEmail = origCommitterEmail;
      committerDateStr = origCommitterDateStr;
      message = origMessage;
    }
  }
</script>

<div class="editor-panel">
  {#if !commit}
    <div class="empty-state">
      <p>Select a commit to edit</p>
    </div>
  {:else}
    <div class="editor-header">
      <span class="oid-label">Commit</span>
      <code class="oid-value">{commit.oid}</code>
      {#if commit.is_merge}
        <span class="merge-badge">Merge</span>
      {/if}
    </div>

    <div class="editor-body">
      <fieldset class="field-group">
        <legend>Author</legend>
        <div class="field">
          <label for="author-name">Name</label>
          <input id="author-name" type="text" bind:value={authorName} />
        </div>
        <div class="field">
          <label for="author-email">Email</label>
          <input id="author-email" type="email" bind:value={authorEmail} />
        </div>
        <div class="field">
          <label for="author-date">Date</label>
          <input id="author-date" type="datetime-local" step="1" bind:value={authorDateStr} />
        </div>
      </fieldset>

      <fieldset class="field-group">
        <legend>Committer</legend>
        <div class="field">
          <label for="committer-name">Name</label>
          <input id="committer-name" type="text" bind:value={committerName} />
        </div>
        <div class="field">
          <label for="committer-email">Email</label>
          <input id="committer-email" type="email" bind:value={committerEmail} />
        </div>
        <div class="field">
          <label for="committer-date">Date</label>
          <input id="committer-date" type="datetime-local" step="1" bind:value={committerDateStr} />
        </div>
      </fieldset>

      <fieldset class="field-group">
        <legend>Message</legend>
        <div class="field message-field">
          <textarea bind:value={message} rows="8"></textarea>
        </div>
      </fieldset>

      {#if commit.parent_oids.length > 0}
        <div class="parents">
          <span class="parents-label">Parents:</span>
          {#each commit.parent_oids as parentOid}
            <code class="parent-oid">{parentOid.slice(0, 7)}</code>
          {/each}
        </div>
      {/if}
    </div>

    <div class="editor-footer">
      <button class="btn btn-secondary" onclick={handleDiscard} disabled={!hasChanges || saving}>
        Discard
      </button>
      <button class="btn btn-primary" onclick={handleSave} disabled={!hasChanges || saving}>
        {saving ? "Saving..." : "Save Changes"}
      </button>
    </div>

    {#if showConfirm}
      <div class="confirm-overlay" role="dialog">
        <div class="confirm-dialog">
          <h3>Confirm History Rewrite</h3>
          <p>
            This will rewrite commit <code>{commit.oid.slice(0, 7)}</code> and
            all its descendants. This operation changes commit hashes and cannot
            be easily undone.
          </p>
          <div class="confirm-actions">
            <button class="btn btn-secondary" onclick={() => (showConfirm = false)}>Cancel</button>
            <button class="btn btn-danger" onclick={confirmSave}>Rewrite History</button>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .editor-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    position: relative;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
  }

  .editor-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border);
  }

  .oid-label {
    color: var(--text-muted);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .oid-value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--accent);
    background: var(--bg-primary);
    padding: 2px 6px;
    border-radius: 3px;
  }

  .merge-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--warning);
    color: var(--bg-primary);
    font-weight: 600;
    text-transform: uppercase;
  }

  .editor-body {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .field-group {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 12px;
  }

  .field-group legend {
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 0 4px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 8px;
  }

  .field:first-child {
    margin-top: 0;
  }

  .field label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .field input,
  .field textarea {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 6px 10px;
    color: var(--text-primary);
    outline: none;
    transition: border-color 0.15s;
  }

  .field input:focus,
  .field textarea:focus {
    border-color: var(--accent);
  }

  .field textarea {
    font-family: var(--font-mono);
    font-size: 12px;
    resize: vertical;
    min-height: 100px;
  }

  .message-field {
    margin-top: 0;
  }

  .parents {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-top: 4px;
  }

  .parents-label {
    color: var(--text-muted);
    font-size: 12px;
  }

  .parent-oid {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    background: var(--bg-surface);
    padding: 2px 6px;
    border-radius: 3px;
  }

  .editor-footer {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    background: var(--bg-surface);
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

  .btn-danger {
    background: var(--danger);
    color: var(--bg-primary);
    border-color: var(--danger);
  }

  .btn-danger:hover:not(:disabled) {
    opacity: 0.9;
  }

  .confirm-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .confirm-dialog {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 24px;
    max-width: 420px;
    width: 90%;
  }

  .confirm-dialog h3 {
    margin-bottom: 12px;
    color: var(--warning);
  }

  .confirm-dialog p {
    color: var(--text-secondary);
    line-height: 1.6;
    margin-bottom: 20px;
  }

  .confirm-dialog code {
    font-family: var(--font-mono);
    background: var(--bg-primary);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 12px;
  }

  .confirm-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
</style>
