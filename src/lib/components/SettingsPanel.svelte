<script lang="ts">
  import type { Settings, Theme } from '../types';
  import { ipc } from '../ipc';
  import { panels } from '../stores/panels.svelte';
  import { theme as themeStore } from '../stores/theme.svelte';
  import { DEFAULT_MEASURE_CH } from '../util/measure';

  let settings: Settings | null = $state(null);
  let saving = $state(false);
  let error: string | null = $state(null);

  async function load() {
    try {
      settings = await ipc.getSettings();
      error = null;
    } catch (e) {
      error = formatError(e);
    }
  }

  $effect(() => {
    if (panels.active === 'settings' && !settings) load();
  });

  // Save settings whenever a field changes — debounced via Svelte's effect graph.
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    const s = settings;
    if (!s) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      saving = true;
      try {
        await ipc.setSettings(s);
        // Apply live settings.
        themeStore.set(s.theme);
        document.documentElement.dataset.dropcap = s.dropCaps ? 'true' : 'false';
        error = null;
      } catch (e) {
        error = formatError(e);
      } finally {
        saving = false;
      }
    }, 200);
  });

  function setTheme(t: Theme) {
    if (!settings) return;
    settings = { ...settings, theme: t };
  }

  function formatError(e: unknown): string {
    if (typeof e === 'object' && e !== null && 'message' in e) {
      return String((e as { message: string }).message);
    }
    return String(e);
  }
</script>

<div class="panel" role="region" aria-labelledby="settings-title">
  <header class="head">
    <h3 id="settings-title">Settings</h3>
    <button type="button" class="close" onclick={panels.close} aria-label="Close">
      ✕
    </button>
  </header>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if settings}
    <div class="section">
      <h4>Theme</h4>
      <div class="segmented">
        <button
          type="button"
          class="seg-btn"
          class:active={settings.theme === 'system'}
          onclick={() => setTheme('system')}
        >System</button>
        <button
          type="button"
          class="seg-btn"
          class:active={settings.theme === 'light'}
          onclick={() => setTheme('light')}
        >Light</button>
        <button
          type="button"
          class="seg-btn"
          class:active={settings.theme === 'dark'}
          onclick={() => setTheme('dark')}
        >Dark</button>
      </div>
    </div>

    <div class="section">
      <h4>Typography</h4>

      <label class="field">
        <span>Measure</span>
        <input
          type="number"
          min="40"
          max="160"
          step="2"
          bind:value={settings.measureCh}
        />
        <span class="unit">ch</span>
        {#if settings.measureCh !== DEFAULT_MEASURE_CH}
          <button
            type="button"
            class="inline-reset"
            onclick={() => {
              if (settings) settings = { ...settings, measureCh: DEFAULT_MEASURE_CH };
            }}
            title="Reset to default ({DEFAULT_MEASURE_CH}ch)"
          >Reset</button>
        {/if}
      </label>
      <p class="field-hint">Drag the column's left or right edge to resize live · double-click an edge to reset.</p>

      <label class="field">
        <span>Scale</span>
        <input
          type="range"
          min="0.85"
          max="1.25"
          step="0.01"
          bind:value={settings.fontScale}
        />
        <span class="unit">{settings.fontScale.toFixed(2)}×</span>
      </label>

      <label class="toggle">
        <input type="checkbox" bind:checked={settings.dropCaps} />
        Drop cap on first paragraph
      </label>
    </div>

    <div class="section">
      <h4>Content</h4>
      <label class="toggle">
        <input type="checkbox" bind:checked={settings.enableMath} />
        Render math (KaTeX)
      </label>
      <label class="toggle">
        <input type="checkbox" bind:checked={settings.enableMermaid} />
        Render mermaid diagrams
      </label>
    </div>

    <div class="section">
      <h4>Trust</h4>
      {#if settings.trustedHosts.length === 0}
        <p class="hint-sub">No trusted hosts yet.</p>
      {:else}
        <ul class="hosts">
          {#each settings.trustedHosts as host (host)}
            <li>
              <span>{host}</span>
              <button
                type="button"
                class="link-btn"
                onclick={() => {
                  if (!settings) return;
                  settings = {
                    ...settings,
                    trustedHosts: settings.trustedHosts.filter((h) => h !== host)
                  };
                }}
              >Remove</button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <footer class="foot">
      {#if saving}<span class="hint-sub">Saving…</span>{/if}
    </footer>
  {:else}
    <p class="hint">Loading…</p>
  {/if}
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    font-family: var(--font-ui);
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--rule);
    position: sticky;
    top: 0;
    background: inherit;
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

  .section {
    padding: 0.75rem 1rem 1rem;
    border-bottom: 1px solid var(--rule);
  }
  .section h4 {
    margin: 0 0 0.625rem;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ink-dim);
  }

  .segmented {
    display: flex;
    gap: 0.25rem;
  }
  .seg-btn {
    flex: 1 1 0;
    padding: 0.375rem 0.5rem;
    border: 1px solid var(--rule);
    border-radius: 4px;
    background: var(--paper);
    color: var(--ink);
    cursor: pointer;
    font: inherit;
  }
  .seg-btn.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
  }

  .field {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
  }
  .field > span:first-child {
    flex: 0 0 5.5rem;
    color: var(--ink-dim);
  }
  .field input[type='number'] {
    flex: 0 0 4rem;
    padding: 0.25rem 0.375rem;
    border: 1px solid var(--rule);
    border-radius: 3px;
    background: var(--paper);
    color: var(--ink);
    font: inherit;
  }
  .field input[type='range'] {
    flex: 1 1 auto;
  }
  .field .unit {
    color: var(--ink-dim);
    font-size: 0.8125rem;
    min-width: 2.5rem;
  }
  .inline-reset {
    font: inherit;
    font-size: 0.75rem;
    padding: 0.125rem 0.5rem;
    border: 1px solid var(--rule);
    border-radius: 3px;
    background: var(--paper);
    color: var(--ink-dim);
    cursor: pointer;
  }
  .inline-reset:hover {
    background: var(--chrome-hover);
    color: var(--ink);
  }
  .field-hint {
    margin: 0.125rem 0 0.5rem;
    font-size: 0.75rem;
    color: var(--ink-dim);
    line-height: 1.4;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    margin-bottom: 0.375rem;
    color: var(--ink);
    cursor: pointer;
  }

  .hosts {
    list-style: none;
    margin: 0;
    padding: 0;
    font-size: 0.8125rem;
  }
  .hosts li {
    display: flex;
    justify-content: space-between;
    padding: 0.25rem 0;
  }
  .link-btn {
    background: none;
    border: 0;
    color: var(--accent);
    cursor: pointer;
    font: inherit;
    text-decoration: underline;
  }

  .foot {
    padding: 0.75rem 1rem;
    color: var(--ink-dim);
    font-size: 0.8125rem;
  }
  .hint,
  .hint-sub {
    color: var(--ink-dim);
    font-size: 0.875rem;
  }
  .hint {
    padding: 2rem 1rem;
    text-align: center;
  }
  .error {
    padding: 0.75rem 1rem;
    color: var(--danger);
    font-size: 0.875rem;
  }
</style>
