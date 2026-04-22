/**
 * Render math placeholders emitted by the Rust pipeline.
 *
 * The pipeline writes `<span|div class="math math-(inline|display)" data-tex="...">`
 * with no children. We walk these, hand each `data-tex` string to KaTeX, and
 * replace the inner HTML.
 *
 * KaTeX is imported lazily — Vite will split it into its own chunk so it's
 * only fetched when a doc actually contains math.
 */
export async function renderMathIn(root: HTMLElement): Promise<void> {
  const nodes = root.querySelectorAll<HTMLElement>(
    '.math[data-tex]:not([data-rendered="true"])'
  );
  if (nodes.length === 0) return;

  const [{ default: katex }] = await Promise.all([
    import('katex'),
    import('katex/dist/katex.min.css')
  ]);

  for (const node of Array.from(nodes)) {
    const tex = node.dataset.tex ?? '';
    const displayMode = node.classList.contains('math-display');
    try {
      katex.render(tex, node, {
        throwOnError: false,
        displayMode,
        trust: false,
        strict: 'warn'
      });
      node.dataset.rendered = 'true';
    } catch (err) {
      node.textContent = tex;
      node.dataset.rendered = 'error';
      console.warn('katex render failed', err);
    }
  }
}
