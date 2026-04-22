<script lang="ts">
  import type { SearchHit, Source } from '../types';
  import { ipc } from '../ipc';
  import { panels } from '../stores/panels.svelte';
  import { session } from '../stores/session.svelte';
  import { debounce } from '../util/debounce';

  interface Props {
    onOpen: (source: Source) => void;
  }

  const { onOpen }: Props = $props();

  let query = $state('');
  let hits: SearchHit[] = $state([]);
  let searching = $state(false);
  let error: string | null = $state(null);
  let input: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (panels.active === 'search') {
      queueMicrotask(() => input?.focus());
    }
  });

  const run = debounce(async (q: string) => {
    if (!q.trim()) {
      hits = [];
      error = null;
      return;
    }
    searching = true;
    try {
      hits = await ipc.searchFolder(q, 30);
      error = null;
    } catch (e) {
      error = formatError(e);
      hits = [];
    } finally {
      searching = false;
    }
  }, 150);

  $effect(() => {
    run(query);
  });

  function formatError(e: unknown): string {
    if (typeof e === 'object' && e !== null && 'message' in e) {
      return String((e as { message: string }).message);
    }
    return String(e);
  }

  function highlight(snippet: string, q: string): string {
    if (!q.trim()) return escape(snippet);
    const terms = q
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'));
    if (terms.length === 0) return escape(snippet);
    const re = new RegExp(`(${terms.join('|')})`, 'gi');
    return escape(snippet).replace(re, '<mark>$1</mark>');
  }

  function escape(s: string): string {
    return s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  function relativize(path: string): string {
    const root = session.tree?.root;
    if (root && path.startsWith(root)) {
      return path.slice(root.length).replace(/^\/+/, '');
    }
    return path;
  }
</script>

<div class="panel" role="region" aria-labelledby="search-title">
  <header class="head">
    <h3 id="search-title">Search</h3>
    <button type="button" class="close" onclick={panels.close} aria-label="Close">
      ✕
    </button>
  </header>

  <div class="search-row">
    <input
      bind:this={input}
      type="search"
      bind:value={query}
      placeholder={session.tree ? 'Search across this folder…' : 'Open a folder first'}
      disabled={!session.tree}
      aria-label="Search query"
    />
  </div>

  {#if error}
    <p class="error">{error}</p>
  {:else if !session.tree}
    <p class="hint">Open a folder (⌘O) to enable full-text search.</p>
  {:else if query && searching}
    <p class="hint">Searching…</p>
  {:else if query && hits.length === 0}
    <p class="hint">No matches for “{query}”.</p>
  {:else if hits.length > 0}
    <ul class="hits">
      {#each hits as h (h.path)}
        <li class="row">
          <button
            type="button"
            class="hit"
            onclick={() => {
              onOpen({ kind: 'localFile', path: h.path });
              panels.close();
            }}
          >
            <span class="title">{h.title}</span>
            <span class="path">{relativize(h.path)}</span>
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            <span class="snippet">{@html highlight(h.snippet, query)}</span>
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
    overflow: hidden;
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

  .search-row {
    padding: 0.75rem 1rem 0.5rem;
  }
  .search-row input {
    width: 100%;
    padding: 0.375rem 0.625rem;
    border: 1px solid var(--rule);
    border-radius: 4px;
    background: var(--paper);
    color: var(--ink);
    font: inherit;
  }
  .search-row input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
    border-color: var(--accent);
  }

  .hits {
    list-style: none;
    margin: 0;
    padding: 0.25rem 0.5rem 1rem;
    overflow-y: auto;
  }
  .row + .row {
    margin-top: 0.125rem;
  }
  .hit {
    width: 100%;
    text-align: left;
    background: none;
    border: 0;
    color: var(--ink);
    padding: 0.5rem 0.625rem;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    font: inherit;
    border-radius: 4px;
  }
  .hit:hover {
    background: var(--chrome-hover);
  }
  .title {
    font-family: var(--font-serif);
    font-size: 0.9375rem;
  }
  .path {
    font-size: 0.7rem;
    color: var(--ink-dim);
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .snippet {
    font-size: 0.8125rem;
    color: var(--ink-dim);
    line-height: 1.45;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .snippet :global(mark) {
    background: rgba(255, 214, 0, 0.45);
    color: inherit;
    border-radius: 2px;
    padding: 0 0.1em;
  }

  .hint {
    padding: 2rem 1rem;
    color: var(--ink-dim);
    text-align: center;
    font-size: 0.875rem;
  }
  .error {
    padding: 1rem;
    color: var(--danger);
    font-size: 0.875rem;
  }
</style>
