<script lang="ts">
  import type { Bookmark, Source } from '../types';
  import { ipc } from '../ipc';
  import { panels } from '../stores/panels.svelte';

  interface Props {
    onOpen: (source: Source) => void;
  }

  const { onOpen }: Props = $props();

  let bookmarks: Bookmark[] = $state([]);
  let loading = $state(false);
  let error: string | null = $state(null);

  async function refresh() {
    loading = true;
    try {
      bookmarks = await ipc.listBookmarks();
      error = null;
    } catch (e) {
      error = formatError(e);
    } finally {
      loading = false;
    }
  }

  // Reload whenever the panel opens.
  $effect(() => {
    if (panels.active === 'bookmarks') refresh();
  });

  async function remove(id: string) {
    try {
      await ipc.removeBookmark(id);
      bookmarks = bookmarks.filter((b) => b.id !== id);
    } catch (e) {
      error = formatError(e);
    }
  }

  function describe(s: Source): string {
    switch (s.kind) {
      case 'localFile':
        return s.path;
      case 'remote':
        return s.url;
      case 'localFolder':
        return s.root;
    }
  }

  function formatError(e: unknown): string {
    if (typeof e === 'object' && e !== null && 'message' in e) {
      return String((e as { message: string }).message);
    }
    return String(e);
  }
</script>

<div class="panel" role="region" aria-labelledby="bookmarks-title">
  <header class="head">
    <h3 id="bookmarks-title">Bookmarks</h3>
    <button type="button" class="close" onclick={panels.close} aria-label="Close">
      ✕
    </button>
  </header>

  {#if error}
    <p class="error">{error}</p>
  {:else if loading}
    <p class="hint">Loading…</p>
  {:else if bookmarks.length === 0}
    <p class="hint">No bookmarks yet. Press <kbd>⌘D</kbd> on a doc to add one.</p>
  {:else}
    <ul class="list">
      {#each bookmarks as b (b.id)}
        <li class="row">
          <button
            type="button"
            class="label"
            onclick={() => {
              onOpen(b.source);
              panels.close();
            }}
          >
            <span class="title">{b.label || describe(b.source)}</span>
            <span class="sub">{describe(b.source)}</span>
          </button>
          <button
            type="button"
            class="remove"
            onclick={() => remove(b.id)}
            aria-label="Remove bookmark"
          >
            ✕
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: var(--font-ui);
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--rule);
  }
  .head h3 {
    margin: 0;
    font-size: 0.8125rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ink-dim);
  }
  .close {
    background: none;
    border: 0;
    color: var(--ink-dim);
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0.25rem 0.5rem;
    border-radius: 3px;
  }
  .close:hover {
    background: var(--chrome-hover);
    color: var(--ink);
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 0.5rem;
    overflow-y: auto;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .row:hover {
    background: var(--chrome-hover);
    border-radius: 4px;
  }
  .label {
    flex: 1 1 auto;
    text-align: left;
    background: none;
    border: 0;
    padding: 0.5rem 0.625rem;
    cursor: pointer;
    color: var(--ink);
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    font: inherit;
  }
  .title {
    font-size: 0.875rem;
    font-weight: 500;
  }
  .sub {
    font-size: 0.75rem;
    color: var(--ink-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 240px;
  }
  .remove {
    background: none;
    border: 0;
    color: var(--ink-dim);
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: 3px;
    opacity: 0;
    transition: opacity 100ms;
  }
  .row:hover .remove {
    opacity: 1;
  }
  .remove:hover {
    color: var(--danger);
    background: var(--chrome-bg);
  }
  .hint {
    padding: 2rem 1rem;
    color: var(--ink-dim);
    text-align: center;
    font-size: 0.875rem;
  }
  .hint kbd {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    padding: 0.1rem 0.35rem;
    border: 1px solid var(--rule);
    border-radius: 3px;
    background: var(--chrome-bg);
  }
  .error {
    padding: 1rem;
    color: var(--danger);
    font-size: 0.875rem;
  }
</style>
