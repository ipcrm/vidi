<script lang="ts">
  import type { RecentFile, Source } from '../types';
  import { ipc } from '../ipc';
  import { panels } from '../stores/panels.svelte';

  interface Props {
    onOpen: (source: Source) => void;
  }

  const { onOpen }: Props = $props();

  let items: RecentFile[] = $state([]);
  let loading = $state(false);

  async function refresh() {
    loading = true;
    try {
      items = await ipc.listRecents();
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (panels.active === 'recents') refresh();
  });

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

  function when(ts: number): string {
    const delta = Math.floor(Date.now() / 1000) - ts;
    if (delta < 60) return 'just now';
    if (delta < 3600) return `${Math.floor(delta / 60)}m ago`;
    if (delta < 86400) return `${Math.floor(delta / 3600)}h ago`;
    return `${Math.floor(delta / 86400)}d ago`;
  }
</script>

<div class="panel" role="region" aria-labelledby="recents-title">
  <header class="head">
    <h3 id="recents-title">Recents</h3>
    <button type="button" class="close" onclick={panels.close} aria-label="Close">
      ✕
    </button>
  </header>

  {#if loading}
    <p class="hint">Loading…</p>
  {:else if items.length === 0}
    <p class="hint">No recent documents yet.</p>
  {:else}
    <ul class="list">
      {#each items as r (r.openedAt + describe(r.source))}
        <li class="row">
          <button
            type="button"
            class="label"
            onclick={() => {
              onOpen(r.source);
              panels.close();
            }}
          >
            <span class="title">{r.title}</span>
            <span class="sub">
              <span class="path">{describe(r.source)}</span>
              <span class="when">{when(r.openedAt)}</span>
            </span>
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
  .row + .row {
    margin-top: 0.125rem;
  }
  .label {
    width: 100%;
    text-align: left;
    background: none;
    border: 0;
    padding: 0.5rem 0.625rem;
    cursor: pointer;
    color: var(--ink);
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    font: inherit;
    border-radius: 4px;
  }
  .label:hover {
    background: var(--chrome-hover);
  }
  .title {
    font-size: 0.9375rem;
    font-family: var(--font-serif);
  }
  .sub {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: var(--ink-dim);
  }
  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1 1 auto;
  }
  .when {
    flex: 0 0 auto;
  }
  .hint {
    padding: 2rem 1rem;
    color: var(--ink-dim);
    text-align: center;
    font-size: 0.875rem;
  }
</style>
