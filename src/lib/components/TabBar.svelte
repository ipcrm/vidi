<script lang="ts">
  import { session, type Tab } from '../stores/session.svelte';
  import { onMount } from 'svelte';

  interface Props {
    onNewTab: () => void;
  }

  const { onNewTab }: Props = $props();

  let bar: HTMLElement | undefined = $state();
  let overflowCount = $state(0);
  let overflowOpen = $state(false);
  let overflowQuery = $state('');
  let overflowInput: HTMLInputElement | undefined = $state();

  // Recompute which tabs fit after any layout change.
  let resizeObserver: ResizeObserver | null = null;
  onMount(() => {
    if (!bar) return;
    resizeObserver = new ResizeObserver(() => measureOverflow());
    resizeObserver.observe(bar);
    // Also observe each child so width changes (from title updates) trigger recompute.
    return () => resizeObserver?.disconnect();
  });

  // Re-measure when the tab list changes.
  $effect(() => {
    // Reactive on session.tabs length + active index.
    void session.tabs.length;
    const activeId = session.activeId;
    queueMicrotask(() => {
      measureOverflow();
      // Scroll active into view so it's never behind the overflow chip.
      const el = bar?.querySelector<HTMLElement>(
        `[data-tab-id="${cssEscape(activeId)}"]`
      );
      el?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    });
  });

  function cssEscape(s: string): string {
    return typeof CSS !== 'undefined' && CSS.escape ? CSS.escape(s) : s;
  }

  function measureOverflow() {
    if (!bar) return;
    const tabs = Array.from(bar.querySelectorAll<HTMLElement>('[data-tab]'));
    const barRight = bar.getBoundingClientRect().right - 72; // reserve room for + and ⌄
    let hidden = 0;
    for (const t of tabs) {
      const r = t.getBoundingClientRect();
      if (r.right > barRight) hidden += 1;
    }
    overflowCount = hidden;
  }

  function filteredTabs(): Tab[] {
    const q = overflowQuery.trim().toLowerCase();
    if (!q) return session.tabs;
    return session.tabs.filter((t) => {
      const title = session.titleForTab(t).toLowerCase();
      const sub = subLabel(t).toLowerCase();
      return title.includes(q) || sub.includes(q);
    });
  }

  function subLabel(t: Tab): string {
    if (t.source?.kind === 'localFile') return t.source.path;
    if (t.source?.kind === 'remote') return t.source.url;
    if (t.tree) return t.tree.root;
    return '';
  }

  function toggleOverflow() {
    overflowOpen = !overflowOpen;
    if (overflowOpen) {
      overflowQuery = '';
      queueMicrotask(() => overflowInput?.focus());
    }
  }

  function pick(id: string) {
    session.activate(id);
    overflowOpen = false;
  }

  function onOverflowKey(ev: KeyboardEvent) {
    if (ev.key === 'Escape') {
      ev.preventDefault();
      overflowOpen = false;
      return;
    }
    if (ev.key === 'Enter') {
      ev.preventDefault();
      const first = filteredTabs()[0];
      if (first) pick(first.id);
    }
  }

  function onTabClose(ev: MouseEvent, id: string) {
    ev.stopPropagation();
    session.closeTab(id);
  }

  function onTabMiddleClick(ev: MouseEvent, id: string) {
    if (ev.button === 1) {
      ev.preventDefault();
      session.closeTab(id);
    }
  }

  function onTabKey(ev: KeyboardEvent, id: string) {
    if (ev.key === 'Enter' || ev.key === ' ') {
      ev.preventDefault();
      session.activate(id);
    }
  }
</script>

<div class="tab-bar" role="tablist" aria-label="Open documents" bind:this={bar}>
  <div class="tab-scroll">
    {#each session.tabs as t (t.id)}
      <div
        role="tab"
        tabindex="0"
        class="tab"
        class:active={t.id === session.activeId}
        class:loading={t.loading}
        data-tab
        data-tab-id={t.id}
        aria-selected={t.id === session.activeId}
        title={session.titleForTab(t)}
        onclick={() => session.activate(t.id)}
        onkeydown={(e) => onTabKey(e, t.id)}
        onauxclick={(e) => onTabMiddleClick(e, t.id)}
      >
        <span class="title">{session.titleForTab(t)}</span>
        {#if session.tabs.length > 1}
          <button
            type="button"
            class="close"
            aria-label="Close tab"
            title="Close tab (⌘W)"
            onclick={(e) => onTabClose(e, t.id)}
            tabindex="-1"
          >✕</button>
        {/if}
      </div>
    {/each}
  </div>

  {#if overflowCount > 0}
    <button
      type="button"
      class="chip chip-overflow"
      onclick={toggleOverflow}
      aria-haspopup="listbox"
      aria-expanded={overflowOpen}
      title="Search {overflowCount} hidden tab{overflowCount === 1 ? '' : 's'}"
      aria-label="Show {overflowCount} hidden tabs"
    >
      <svg
        width="10"
        height="10"
        viewBox="0 0 10 10"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M2.5 3.5 L5 6 L7.5 3.5" />
      </svg>
      <span class="chip-count">{overflowCount}</span>
    </button>
  {/if}

  <button
    type="button"
    class="chip chip-plus"
    onclick={onNewTab}
    aria-label="New tab"
    title="New tab (⌘T)"
  >
    <svg
      width="12"
      height="12"
      viewBox="0 0 12 12"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      aria-hidden="true"
    >
      <line x1="6" y1="2" x2="6" y2="10" />
      <line x1="2" y1="6" x2="10" y2="6" />
    </svg>
  </button>

  {#if overflowOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="overflow-scrim" onclick={() => (overflowOpen = false)}></div>
    <div class="overflow-pop" role="dialog" aria-label="Find open tab">
      <input
        bind:this={overflowInput}
        bind:value={overflowQuery}
        onkeydown={onOverflowKey}
        placeholder="Search {session.tabs.length} open tabs…"
        aria-label="Search tabs"
      />
      <ul class="pop-list">
        {#each filteredTabs() as t (t.id)}
          <li>
            <button
              type="button"
              class="pop-item"
              class:active={t.id === session.activeId}
              onclick={() => pick(t.id)}
            >
              <span class="pop-title">{session.titleForTab(t)}</span>
              {#if subLabel(t)}
                <span class="pop-sub">{subLabel(t)}</span>
              {/if}
            </button>
          </li>
        {/each}
        {#if filteredTabs().length === 0}
          <li class="pop-empty">No matching tabs</li>
        {/if}
      </ul>
    </div>
  {/if}
</div>

<style>
  .tab-bar {
    position: relative;
    display: flex;
    align-items: stretch;
    gap: 0;
    padding: 0.25rem 0.375rem 0;
    background: var(--chrome-bg);
    border-bottom: 1px solid var(--rule);
    font-family: var(--font-ui);
    font-size: 0.8125rem;
    min-height: 34px;
    flex: 0 0 auto;
  }

  .tab-scroll {
    flex: 1 1 auto;
    display: flex;
    gap: 0.125rem;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .tab-scroll::-webkit-scrollbar {
    display: none;
  }

  .tab {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    max-width: 220px;
    min-width: 72px;
    padding: 0.25rem 0.5rem 0.25rem 0.625rem;
    background: transparent;
    color: var(--ink-dim);
    border: 1px solid transparent;
    border-bottom: 0;
    border-top-left-radius: 5px;
    border-top-right-radius: 5px;
    cursor: pointer;
    font: inherit;
    position: relative;
    top: 1px;
    white-space: nowrap;
  }
  .tab:hover {
    color: var(--ink);
    background: var(--chrome-hover);
  }
  .tab.active {
    color: var(--ink);
    background: var(--paper);
    border-color: var(--rule);
    border-bottom-color: var(--paper);
    z-index: 1;
  }
  .tab.loading .title::after {
    content: '';
    display: inline-block;
    width: 6px;
    height: 6px;
    margin-left: 0.4em;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 1s infinite;
    vertical-align: middle;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.35; }
    50% { opacity: 1; }
  }
  .title {
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .close {
    width: 18px;
    height: 18px;
    background: none;
    border: 0;
    color: var(--ink-dim);
    border-radius: 3px;
    cursor: pointer;
    line-height: 1;
    padding: 0;
    font-size: 0.75rem;
    opacity: 0.6;
  }
  .tab:hover .close,
  .tab.active .close {
    opacity: 1;
  }
  .close:hover {
    background: var(--chrome-hover);
    color: var(--ink);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    min-width: 28px;
    padding: 0 0.5rem;
    margin-left: 0.125rem;
    border: 0;
    background: none;
    color: var(--ink-dim);
    cursor: pointer;
    border-radius: 4px;
    font: inherit;
    font-size: 0.75rem;
    line-height: 1;
    height: 26px;
    align-self: center;
    flex: 0 0 auto;
    white-space: nowrap;
  }
  .chip:hover {
    background: var(--chrome-hover);
    color: var(--ink);
  }
  .chip-count {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  .overflow-scrim {
    position: fixed;
    inset: 0;
    z-index: 50;
    background: transparent;
  }
  .overflow-pop {
    position: absolute;
    top: calc(100% + 4px);
    right: 0.5rem;
    width: min(360px, calc(100vw - 2rem));
    background: var(--paper);
    border: 1px solid var(--rule);
    border-radius: 6px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.18);
    z-index: 51;
    display: flex;
    flex-direction: column;
    max-height: 60vh;
    overflow: hidden;
  }
  .overflow-pop input {
    padding: 0.5rem 0.75rem;
    border: 0;
    border-bottom: 1px solid var(--rule);
    background: var(--paper);
    color: var(--ink);
    font: inherit;
    outline: none;
  }
  .pop-list {
    list-style: none;
    margin: 0;
    padding: 0.25rem;
    overflow-y: auto;
  }
  .pop-item {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    background: none;
    border: 0;
    color: var(--ink);
    text-align: left;
    padding: 0.4rem 0.625rem;
    cursor: pointer;
    border-radius: 4px;
    font: inherit;
  }
  .pop-item:hover,
  .pop-item.active {
    background: var(--chrome-hover);
  }
  .pop-item.active {
    outline: 1px solid var(--accent);
  }
  .pop-title {
    font-size: 0.875rem;
  }
  .pop-sub {
    font-size: 0.75rem;
    color: var(--ink-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
  }
  .pop-empty {
    padding: 1rem;
    color: var(--ink-dim);
    text-align: center;
    font-size: 0.875rem;
  }
</style>
