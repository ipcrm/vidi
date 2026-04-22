<script lang="ts">
  import { confirm } from '../stores/confirm.svelte';

  let trustHost = $state(false);
  let firstButton: HTMLButtonElement | undefined = $state();

  // Reset + focus when a request arrives.
  $effect(() => {
    if (confirm.request) {
      trustHost = false;
      queueMicrotask(() => firstButton?.focus());
    }
  });

  function onKey(ev: KeyboardEvent) {
    if (ev.key === 'Escape') {
      ev.preventDefault();
      confirm.cancel();
    }
  }

  function onApprove() {
    confirm.approve(trustHost);
  }
</script>

<svelte:window onkeydown={confirm.request ? onKey : undefined} />

{#if confirm.request}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="scrim" onclick={confirm.cancel} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="confirm-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 id="confirm-title" class="title">
        {#if confirm.request.kind === 'image'}
          Load image from {confirm.request.host}?
        {:else}
          Open external link?
        {/if}
      </h2>
      <p class="url">{confirm.request.url}</p>
      {#if confirm.request.kind === 'link'}
        <p class="sub">This will open in your default browser.</p>
      {/if}

      <label class="trust">
        <input type="checkbox" bind:checked={trustHost} />
        Trust <code>{confirm.request.host}</code> for future loads
      </label>

      <div class="actions">
        <button type="button" class="btn" onclick={confirm.cancel}>Cancel</button>
        <button
          bind:this={firstButton}
          type="button"
          class="btn btn-primary"
          onclick={onApprove}
        >
          {confirm.request.kind === 'image' ? 'Load image' : 'Open link'}
        </button>
      </div>
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
    padding: 1.25rem 1.25rem 1rem;
    width: min(440px, 92vw);
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.25);
    font-family: var(--font-ui, system-ui);
  }
  .title {
    margin: 0 0 0.5rem;
    font-size: 1.0625rem;
    font-weight: 600;
  }
  .url {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.8125rem;
    overflow-wrap: anywhere;
    background: var(--code-bg);
    padding: 0.5rem 0.625rem;
    border-radius: 4px;
    margin: 0.25rem 0 0.75rem;
  }
  .sub {
    font-size: 0.8125rem;
    color: var(--ink-dim);
    margin: 0 0 0.75rem;
  }
  .trust {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: var(--ink-dim);
    margin-bottom: 1rem;
  }
  .trust code {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .btn {
    font: inherit;
    padding: 0.5rem 0.875rem;
    border: 1px solid var(--rule);
    border-radius: 4px;
    background: var(--paper);
    color: var(--ink);
    cursor: pointer;
  }
  .btn:hover {
    background: var(--chrome-hover);
  }
  .btn-primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
  }
  .btn-primary:hover {
    filter: brightness(1.08);
  }
  .btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
