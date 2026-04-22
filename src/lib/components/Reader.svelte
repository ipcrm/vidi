<script lang="ts">
  import type { RenderedDoc, Source } from '../types';
  import { ipc } from '../ipc';
  import { session } from '../stores/session.svelte';
  import { confirm, hostOf } from '../stores/confirm.svelte';
  import { renderMathIn } from '../render/katex-client';
  import { renderMermaidIn } from '../render/mermaid';
  import {
    DEFAULT_MEASURE_CH,
    applyMeasureCh,
    clampMeasureCh,
    measureChPx,
    persistMeasureCh,
    readMeasureCh
  } from '../util/measure';

  interface Props {
    doc: RenderedDoc;
    onNavigate: (source: Source) => void;
  }

  const { doc, onNavigate }: Props = $props();

  let container: HTMLElement | undefined = $state();

  // --- Drag-to-resize state ---------------------------------------------
  let dragging = $state(false);
  let dragStartX = 0;
  let dragStartMeasureCh = 0;
  let dragSign: 1 | -1 = 1;
  let chPx = 9;

  function onHandleDown(ev: PointerEvent, sign: 1 | -1) {
    if (!container) return;
    ev.preventDefault();
    (ev.currentTarget as HTMLElement).setPointerCapture(ev.pointerId);
    dragging = true;
    dragSign = sign;
    dragStartX = ev.clientX;
    dragStartMeasureCh = readMeasureCh();
    chPx = measureChPx(container);
    window.addEventListener('pointermove', onDragMove);
    window.addEventListener('pointerup', onDragUp, { once: true });
    window.addEventListener('pointercancel', onDragUp, { once: true });
  }

  function onDragMove(ev: PointerEvent) {
    if (!dragging) return;
    const dx = (ev.clientX - dragStartX) * dragSign;
    // Column is centered, so both edges shift by the same amount — a drag
    // of `dx` widens the column by `2·dx`.
    const next = clampMeasureCh(dragStartMeasureCh + (dx * 2) / chPx);
    applyMeasureCh(next);
  }

  function onDragUp() {
    if (!dragging) return;
    dragging = false;
    window.removeEventListener('pointermove', onDragMove);
    persistMeasureCh(readMeasureCh());
  }

  function resetMeasure() {
    applyMeasureCh(DEFAULT_MEASURE_CH);
    persistMeasureCh(DEFAULT_MEASURE_CH);
  }

  /** Track cursor Y relative to the prose so BOTH column-edge bars can
   *  follow it in sync — not just the handle the cursor is over.
   *
   *  We write the value directly onto each handle (as an integer pixel)
   *  to avoid any subpixel rounding drift between the two `::before`s. */
  function onHandleMove(ev: PointerEvent) {
    if (!container) return;
    const rect = container.getBoundingClientRect();
    const y = Math.round(ev.clientY - rect.top);
    const val = `${y}px`;
    container
      .querySelectorAll<HTMLElement>('.measure-handle')
      .forEach((h) => h.style.setProperty('--hover-y', val));
  }

  // Re-render math/mermaid + swap auto-trusted image placeholders whenever
  // the document HTML changes.
  $effect(() => {
    if (!container) return;
    // Reset — container innerHTML is driven by {@html}; just run hydrators.
    if (doc.hasMath) renderMathIn(container);
    if (doc.hasMermaid) renderMermaidIn(container);
    autoLoadTrustedImages(container);
    wireImageFallbacks(container);
  });

  function wireImageFallbacks(root: HTMLElement) {
    const imgs = root.querySelectorAll<HTMLImageElement>('img:not([data-wired])');
    imgs.forEach((img) => {
      img.dataset.wired = 'true';
      const handler = () => {
        const src = img.getAttribute('src') ?? '';
        let display = src;
        if (src.startsWith('asset://localhost/')) {
          try {
            display = decodeURIComponent(src.slice('asset://localhost/'.length));
          } catch {
            // leave as-is
          }
        }
        console.warn('[visum] image failed to load', {
          src,
          decoded: display,
          alt: img.alt,
          outerHTML: img.outerHTML,
          attributeNames: img.getAttributeNames()
        });
        const span = document.createElement('span');
        span.className = 'image-broken';
        span.setAttribute('role', 'img');
        span.setAttribute('aria-label', img.alt || 'broken image');
        span.title = `Couldn't load: ${display}`;
        const label = document.createElement('span');
        label.className = 'image-broken-label';
        label.textContent = img.alt || 'Image failed to load';
        const path = document.createElement('code');
        path.className = 'image-broken-path';
        path.textContent = display;
        span.appendChild(label);
        span.appendChild(path);
        img.replaceWith(span);
      };
      // If it already failed (cached from a previous render), swap immediately.
      if (img.complete && img.naturalWidth === 0) {
        handler();
      } else {
        img.addEventListener('error', handler, { once: true });
      }
    });
  }

  function autoLoadTrustedImages(root: HTMLElement) {
    const placeholders = root.querySelectorAll<HTMLElement>('.image-placeholder');
    placeholders.forEach((ph) => {
      const url = ph.dataset.src;
      if (!url) return;
      if (session.isHostTrusted(hostOf(url))) {
        swapPlaceholderToImg(ph);
      }
    });
  }

  function swapPlaceholderToImg(ph: HTMLElement) {
    const url = ph.dataset.src;
    if (!url) return;
    const alt = ph.querySelector('.image-placeholder-alt')?.textContent ?? '';
    const img = document.createElement('img');
    img.src = url;
    img.alt = alt;
    img.loading = 'lazy';
    img.className = 'loaded-image';
    ph.replaceWith(img);
  }

  async function onClick(ev: MouseEvent) {
    const target = ev.target as HTMLElement | null;
    if (!target) return;

    // Image placeholder click — confirm then load.
    const placeholder = target.closest<HTMLElement>('.image-placeholder');
    if (placeholder) {
      ev.preventDefault();
      const url = placeholder.dataset.src;
      if (!url) return;
      const host = hostOf(url);
      if (session.isHostTrusted(host)) {
        swapPlaceholderToImg(placeholder);
        return;
      }
      const { approved, trustHost } = await confirm.show({
        kind: 'image',
        url,
        host
      });
      if (approved) {
        if (trustHost) session.trustHost(host);
        swapPlaceholderToImg(placeholder);
      }
      return;
    }

    // Link click.
    const link = target.closest<HTMLAnchorElement>('a');
    if (!link) return;

    // Internal markdown link.
    if (link.dataset.internal === 'true') {
      ev.preventDefault();
      const payload = link.dataset.resolved;
      if (!payload) return;
      try {
        const source = JSON.parse(payload) as Source;
        onNavigate(source);
      } catch {
        // Resolved payload malformed — ignore.
      }
      return;
    }

    // External link.
    if (link.dataset.external === 'true') {
      ev.preventDefault();
      const url = link.href;
      const host = hostOf(url);
      if (session.isHostTrusted(host)) {
        await ipc.openExternal(url);
        return;
      }
      const { approved, trustHost } = await confirm.show({
        kind: 'link',
        url,
        host
      });
      if (approved) {
        if (trustHost) session.trustHost(host);
        await ipc.openExternal(url);
      }
      return;
    }

    // Anchor-only link — scroll inside the reader container (default
    // fragment navigation doesn't reliably scroll nested scroll containers).
    const href = link.getAttribute('href') ?? '';
    if (href.startsWith('#') && href.length > 1 && container) {
      ev.preventDefault();
      const id = decodeURIComponent(href.slice(1));
      const target = container.querySelector<HTMLElement>(
        `[id="${cssAttrEscape(id)}"]`
      );
      if (target) {
        target.scrollIntoView({ block: 'start', behavior: 'smooth' });
      }
    }
  }

  function cssAttrEscape(s: string): string {
    return s.replace(/["\\]/g, '\\$&');
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<article class="prose" onclick={onClick} bind:this={container} class:dragging>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="measure-handle measure-handle-l"
    onpointerdown={(e) => onHandleDown(e, -1)}
    onpointermove={onHandleMove}
    ondblclick={resetMeasure}
    role="separator"
    aria-orientation="vertical"
    aria-label="Drag to resize column (double-click to reset)"
    title="Drag to resize · double-click to reset"
  ></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="measure-handle measure-handle-r"
    onpointerdown={(e) => onHandleDown(e, 1)}
    onpointermove={onHandleMove}
    ondblclick={resetMeasure}
    role="separator"
    aria-orientation="vertical"
    aria-label="Drag to resize column (double-click to reset)"
    title="Drag to resize · double-click to reset"
  ></div>

  {#if doc.title}
    <header class="doc-header">
      <p class="doc-eyebrow">
        {doc.wordCount.toLocaleString()} words
        · {doc.toc.length.toLocaleString()} sections
      </p>
    </header>
  {/if}
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  {@html doc.html}
</article>

<style>
  .prose {
    max-width: var(--measure);
    margin: 0 auto;
    padding: 3rem 1.25rem 6rem;
    position: relative;
  }
  .prose.dragging {
    user-select: none;
  }

  .doc-header {
    margin-bottom: 0.75rem;
  }

  .doc-eyebrow {
    font-family: var(--font-ui, system-ui);
    font-size: 0.75rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-dim);
    margin: 0;
  }

  /* Drag handles — invisible hitbox; accent rule follows cursor on hover. */
  .measure-handle {
    position: absolute;
    top: 0;
    height: 100%;
    width: 18px;
    cursor: ew-resize;
    touch-action: none;
    z-index: 2;
  }
  .measure-handle-l {
    left: -14px;
  }
  .measure-handle-r {
    right: -14px;
  }

  /* Short vertical rule that follows the cursor while hovering the handle.
     --hover-y is updated by onpointermove to reflect the cursor's y offset
     within the handle. */
  .measure-handle::before {
    content: '';
    position: absolute;
    left: 50%;
    top: var(--hover-y, 50%);
    transform: translate(-50%, -50%);
    width: 2px;
    height: 25vh;
    background: var(--accent);
    border-radius: 2px;
    opacity: 0;
    transition:
      opacity 120ms ease,
      width 120ms ease;
  }

  /* When any handle is hovered, BOTH column-edge bars show in sync.
     :has() lets us reflect the hover state of one handle on both. */
  .prose:has(.measure-handle:hover) .measure-handle::before,
  .prose.dragging .measure-handle::before {
    opacity: 0.35;
  }
  .prose.dragging .measure-handle::before {
    opacity: 0.55;
    width: 3px;
  }
</style>
