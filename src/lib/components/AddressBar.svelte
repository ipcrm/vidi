<script lang="ts">
  interface Props {
    onOpenUrl: (url: string) => void;
    onOpenFolder: () => void;
    onBack: () => void;
    onForward: () => void;
    canGoBack: boolean;
    canGoForward: boolean;
    title: string | null;
    busy: boolean;
  }

  const {
    onOpenUrl,
    onOpenFolder,
    onBack,
    onForward,
    canGoBack,
    canGoForward,
    title,
    busy
  }: Props = $props();
  let url = $state('');

  function submit(ev: SubmitEvent) {
    ev.preventDefault();
    const trimmed = url.trim();
    if (!trimmed) return;
    onOpenUrl(trimmed);
  }
</script>

<header class="address-bar">
  <div class="brand" aria-hidden="true">
    <span class="logo-mark">V</span>
    <span class="logo-word">Vidi</span>
  </div>

  <div class="nav-arrows" role="group" aria-label="Navigation">
    <button
      type="button"
      class="arrow"
      onclick={onBack}
      disabled={!canGoBack}
      aria-label="Go back"
      title="Back (⌘[)"
    >
      <svg
        viewBox="0 0 16 16"
        width="14"
        height="14"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M10 3 L5 8 L10 13" />
      </svg>
    </button>
    <button
      type="button"
      class="arrow"
      onclick={onForward}
      disabled={!canGoForward}
      aria-label="Go forward"
      title="Forward (⌘])"
    >
      <svg
        viewBox="0 0 16 16"
        width="14"
        height="14"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M6 3 L11 8 L6 13" />
      </svg>
    </button>
  </div>

  <form class="url-form" onsubmit={submit}>
    <input
      type="url"
      class="url-input"
      placeholder="Open URL — github.com/owner/repo, raw URL, or gist…"
      bind:value={url}
      aria-label="URL"
    />
    <button type="submit" class="btn" disabled={busy}>Open</button>
  </form>

  <button type="button" class="btn" onclick={onOpenFolder}>Open folder…</button>

  {#if title}
    <div class="title-display" title={title}>{title}</div>
  {/if}
</header>

<style>
  .address-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--rule);
    background: var(--chrome-bg);
    font-family: var(--font-ui, system-ui);
    font-size: 0.875rem;
    flex: 0 0 auto;
  }

  .brand {
    display: flex;
    align-items: baseline;
    gap: 0.375rem;
    padding: 0 0.25rem 0 0.125rem;
    user-select: none;
  }
  .logo-mark {
    width: 20px;
    height: 20px;
    display: inline-grid;
    place-items: center;
    background: var(--ink);
    color: var(--paper);
    border-radius: 3px;
    font-family: 'Source Serif 4', Georgia, serif;
    font-weight: 700;
    font-size: 0.875rem;
    line-height: 1;
  }
  .logo-word {
    font-family: 'Source Serif 4', Georgia, serif;
    font-size: 1rem;
    letter-spacing: 0.02em;
    color: var(--ink);
  }

  .nav-arrows {
    display: flex;
    gap: 0.125rem;
    flex: 0 0 auto;
  }
  .arrow {
    width: 26px;
    height: 26px;
    display: grid;
    place-items: center;
    border: 1px solid var(--rule);
    background: var(--paper);
    color: var(--ink);
    border-radius: 4px;
    cursor: pointer;
    padding: 0;
  }
  .arrow:hover:not(:disabled) {
    background: var(--chrome-hover);
    color: var(--accent);
  }
  .arrow:disabled {
    color: var(--ink-dim);
    opacity: 0.4;
    cursor: default;
  }
  .arrow:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .url-form {
    flex: 1 1 auto;
    display: flex;
    gap: 0.25rem;
    max-width: 720px;
  }

  .url-input {
    flex: 1 1 auto;
    padding: 0.375rem 0.625rem;
    border: 1px solid var(--rule);
    border-radius: 4px;
    background: var(--paper);
    color: var(--ink);
    font: inherit;
  }
  .url-input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
    border-color: var(--accent);
  }

  .btn {
    font: inherit;
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--rule);
    border-radius: 4px;
    background: var(--paper);
    color: var(--ink);
    cursor: pointer;
    white-space: nowrap;
  }
  .btn:hover {
    background: var(--chrome-hover);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .title-display {
    font-family: 'Source Serif 4', Georgia, serif;
    color: var(--ink-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0 0.5rem;
    border-left: 1px solid var(--rule);
    flex: 0 1 auto;
    min-width: 0;
  }
</style>
