<script lang="ts">
  interface Props {
    container: HTMLElement | undefined;
    open: boolean;
    onClose: () => void;
  }

  const { container, open, onClose }: Props = $props();

  let query = $state('');
  let ranges: Range[] = $state([]);
  let active = $state(0);
  let input: HTMLInputElement | undefined = $state();

  // Focus the input whenever the bar opens.
  $effect(() => {
    if (open) {
      queueMicrotask(() => input?.focus());
    } else {
      clear();
    }
  });

  // Re-run search when query or container content changes.
  $effect(() => {
    if (!open) return;
    runSearch();
  });

  function clear() {
    ranges = [];
    active = 0;
    query = '';
    if (container) {
      container.querySelectorAll('mark.find-match').forEach((m) => {
        const parent = m.parentNode;
        while (m.firstChild) parent?.insertBefore(m.firstChild, m);
        parent?.removeChild(m);
        parent?.normalize();
      });
    }
  }

  function runSearch() {
    // Tear down previous highlights.
    if (!container) {
      ranges = [];
      return;
    }
    container.querySelectorAll('mark.find-match').forEach((m) => {
      const parent = m.parentNode;
      while (m.firstChild) parent?.insertBefore(m.firstChild, m);
      parent?.removeChild(m);
      parent?.normalize();
    });
    ranges = [];
    active = 0;

    if (!query || query.length < 2) return;

    // Walk text nodes and collect match ranges.
    const needle = query.toLowerCase();
    const walker = document.createTreeWalker(
      container,
      NodeFilter.SHOW_TEXT,
      {
        acceptNode(node) {
          if (!node.nodeValue) return NodeFilter.FILTER_REJECT;
          const p = (node as Text).parentElement;
          if (!p) return NodeFilter.FILTER_REJECT;
          if (p.closest('script, style, mark.find-match')) {
            return NodeFilter.FILTER_REJECT;
          }
          return NodeFilter.FILTER_ACCEPT;
        }
      }
    );

    const hits: { node: Text; start: number; end: number }[] = [];
    let n: Text | null;
    while ((n = walker.nextNode() as Text | null)) {
      const text = n.nodeValue ?? '';
      const lower = text.toLowerCase();
      let i = 0;
      while (i < lower.length) {
        const idx = lower.indexOf(needle, i);
        if (idx === -1) break;
        hits.push({ node: n, start: idx, end: idx + needle.length });
        i = idx + needle.length;
      }
    }

    // Wrap each hit with a <mark.find-match>. Wrap back-to-front to avoid
    // indices shifting as we split nodes.
    hits.sort((a, b) => {
      if (a.node !== b.node) return 0; // order per-node; stable enough.
      return b.start - a.start;
    });

    const newRanges: Range[] = [];
    for (const h of hits) {
      const range = document.createRange();
      range.setStart(h.node, h.start);
      range.setEnd(h.node, h.end);
      const mark = document.createElement('mark');
      mark.className = 'find-match';
      try {
        range.surroundContents(mark);
        const r = document.createRange();
        r.selectNodeContents(mark);
        newRanges.push(r);
      } catch {
        // Range could not be surrounded (spans elements); skip.
      }
    }
    // Final ranges left-to-right by document order.
    newRanges.sort((a, b) => {
      const pos = a.compareBoundaryPoints(Range.START_TO_START, b);
      return pos;
    });
    ranges = newRanges;
    if (ranges.length > 0) focusActive();
  }

  function focusActive() {
    if (ranges.length === 0) return;
    active = ((active % ranges.length) + ranges.length) % ranges.length;
    // Scroll the active mark into view.
    const r = ranges[active];
    const el = r.startContainer.parentElement;
    if (el) {
      container
        ?.querySelectorAll('mark.find-match.active')
        .forEach((m) => m.classList.remove('active'));
      el.classList.add('active');
      el.scrollIntoView({ block: 'center', behavior: 'smooth' });
    }
  }

  function next() {
    if (ranges.length === 0) return;
    active = (active + 1) % ranges.length;
    focusActive();
  }

  function prev() {
    if (ranges.length === 0) return;
    active = (active - 1 + ranges.length) % ranges.length;
    focusActive();
  }

  function onKey(ev: KeyboardEvent) {
    if (ev.key === 'Escape') {
      ev.preventDefault();
      onClose();
    } else if (ev.key === 'Enter') {
      ev.preventDefault();
      if (ev.shiftKey) prev();
      else next();
    }
  }
</script>

{#if open}
  <div class="findbar" role="search">
    <input
      bind:this={input}
      type="search"
      bind:value={query}
      placeholder="Find in document…"
      onkeydown={onKey}
      aria-label="Find in document"
    />
    <span class="count">
      {#if query.length < 2}
        —
      {:else}
        {ranges.length === 0 ? '0 matches' : `${active + 1} / ${ranges.length}`}
      {/if}
    </span>
    <button type="button" onclick={prev} aria-label="Previous match">‹</button>
    <button type="button" onclick={next} aria-label="Next match">›</button>
    <button type="button" onclick={onClose} aria-label="Close find">✕</button>
  </div>
{/if}

<style>
  .findbar {
    position: fixed;
    top: 3rem;
    right: 1rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.375rem 0.5rem;
    background: var(--paper);
    border: 1px solid var(--rule);
    border-radius: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
    z-index: 50;
    font-family: var(--font-ui);
    font-size: 0.875rem;
  }
  .findbar input {
    width: 220px;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--rule);
    border-radius: 4px;
    background: var(--paper);
    color: var(--ink);
    font: inherit;
  }
  .findbar .count {
    min-width: 5.5rem;
    text-align: center;
    font-variant-numeric: tabular-nums;
    color: var(--ink-dim);
    font-size: 0.8125rem;
  }
  .findbar button {
    background: none;
    border: 0;
    color: var(--ink-dim);
    cursor: pointer;
    font-size: 1rem;
    padding: 0 0.25rem;
    border-radius: 3px;
  }
  .findbar button:hover {
    background: var(--chrome-hover);
    color: var(--ink);
  }

  :global(mark.find-match) {
    background: rgba(255, 214, 0, 0.45);
    color: inherit;
    border-radius: 2px;
    padding: 0 0.05em;
  }
  :global(mark.find-match.active) {
    background: rgba(255, 165, 0, 0.8);
    outline: 1px solid rgba(0, 0, 0, 0.2);
  }
</style>
