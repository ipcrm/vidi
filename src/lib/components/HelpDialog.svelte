<script lang="ts">
  interface Props {
    open: boolean;
    onClose: () => void;
  }

  const { open, onClose }: Props = $props();

  let firstFocus: HTMLButtonElement | undefined = $state();

  $effect(() => {
    if (open) {
      queueMicrotask(() => firstFocus?.focus());
    }
  });

  function onKey(ev: KeyboardEvent) {
    if (ev.key === 'Escape') {
      ev.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={open ? onKey : undefined} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="scrim" onclick={onClose} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="help-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
    >
      <header class="head">
        <h2 id="help-title">Keyboard shortcuts</h2>
        <button
          bind:this={firstFocus}
          type="button"
          class="close"
          onclick={onClose}
          aria-label="Close help"
        >✕</button>
      </header>

      <div class="body">
        <dl class="group">
          <h3>Open</h3>
          <dt><kbd>⌘O</kbd></dt><dd>Open folder…</dd>
        </dl>

        <dl class="group">
          <h3>Tabs</h3>
          <dt><kbd>⌘T</kbd></dt><dd>New tab</dd>
          <dt><kbd>⌘W</kbd></dt><dd>Close current tab</dd>
          <dt><kbd>⌘⇥</kbd></dt><dd>Next tab</dd>
          <dt><kbd>⌘⇧⇥</kbd></dt><dd>Previous tab</dd>
          <dt><kbd>⌘1</kbd>…<kbd>⌘9</kbd></dt><dd>Jump to tab N</dd>
        </dl>

        <dl class="group">
          <h3>Navigation</h3>
          <dt><kbd>⌘[</kbd></dt><dd>Back</dd>
          <dt><kbd>⌘]</kbd></dt><dd>Forward</dd>
        </dl>

        <dl class="group">
          <h3>Layout</h3>
          <dt><kbd>⌘\</kbd></dt><dd>Toggle sidebar</dd>
        </dl>

        <dl class="group">
          <h3>Search</h3>
          <dt><kbd>⌘F</kbd></dt><dd>Find in document</dd>
          <dt><kbd>⌘⇧F</kbd></dt><dd>Search across folder</dd>
        </dl>

        <dl class="group">
          <h3>Bookmarks</h3>
          <dt><kbd>⌘D</kbd></dt><dd>Bookmark current document</dd>
          <dt><kbd>⌘B</kbd></dt><dd>Bookmarks panel</dd>
          <dt><kbd>⌘Y</kbd></dt><dd>Recents panel</dd>
        </dl>

        <dl class="group">
          <h3>Other</h3>
          <dt><kbd>⌘,</kbd></dt><dd>Settings</dd>
          <dt><kbd>⌘P</kbd></dt><dd>Print / save as PDF</dd>
          <dt><kbd>⌘/</kbd></dt><dd>Show this help</dd>
          <dt><kbd>Esc</kbd></dt><dd>Close dialog</dd>
        </dl>
      </div>

      <footer class="foot">
        <span>Press <kbd>⌘/</kbd> any time to reopen this.</span>
      </footer>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: grid;
    place-items: center;
    z-index: 100;
  }
  .dialog {
    background: var(--paper);
    color: var(--ink);
    border: 1px solid var(--rule);
    border-radius: 8px;
    width: min(640px, 92vw);
    max-height: 86vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.25);
    font-family: var(--font-ui, system-ui);
    overflow: hidden;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.875rem 1.125rem;
    border-bottom: 1px solid var(--rule);
  }
  .head h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    font-family: var(--font-serif, Georgia, serif);
    color: var(--ink);
  }
  .close {
    background: none;
    border: 0;
    color: var(--ink-dim);
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: 3px;
    font-size: 0.9rem;
  }
  .close:hover {
    background: var(--chrome-hover);
    color: var(--ink);
  }

  .body {
    overflow-y: auto;
    padding: 1rem 1.125rem 0.5rem;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem 1.75rem;
  }
  @media (max-width: 600px) {
    .body { grid-template-columns: 1fr; }
  }

  .group {
    margin: 0;
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.35rem 0.75rem;
    align-items: center;
  }
  .group h3 {
    grid-column: 1 / -1;
    margin: 0 0 0.15rem;
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-dim);
  }
  .group dt {
    justify-self: start;
    white-space: nowrap;
  }
  .group dd {
    margin: 0;
    color: var(--ink);
    font-size: 0.85rem;
  }

  kbd {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.72rem;
    padding: 0.12rem 0.4rem;
    border: 1px solid var(--rule);
    border-radius: 3px;
    background: var(--paper-alt, var(--paper));
    color: var(--ink);
    margin-right: 0.1rem;
    white-space: nowrap;
  }

  .foot {
    padding: 0.625rem 1.125rem;
    border-top: 1px solid var(--rule);
    background: var(--chrome-bg);
    color: var(--ink-dim);
    font-size: 0.8125rem;
  }
</style>
