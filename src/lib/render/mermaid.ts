/**
 * Render `<div class="mermaid-source" data-code="<base64>">` placeholders.
 *
 * Mermaid is imported lazily and initialised with `securityLevel: "strict"`
 * to refuse arbitrary HTML in diagrams. Output SVG is passed through
 * DOMPurify as a belt-and-suspenders guard before injecting.
 */

let initPromise: Promise<typeof import('mermaid').default> | null = null;

async function loadMermaid() {
  if (!initPromise) {
    initPromise = import('mermaid').then(({ default: mermaid }) => {
      mermaid.initialize({
        startOnLoad: false,
        securityLevel: 'strict',
        theme: 'default'
      });
      return mermaid;
    });
  }
  return initPromise;
}

export async function renderMermaidIn(root: HTMLElement): Promise<void> {
  const nodes = root.querySelectorAll<HTMLElement>(
    '.mermaid-source[data-code]:not([data-rendered="true"])'
  );
  if (nodes.length === 0) return;

  const [mermaid, { default: DOMPurify }] = await Promise.all([
    loadMermaid(),
    import('dompurify')
  ]);

  let i = 0;
  for (const node of Array.from(nodes)) {
    i += 1;
    const code = b64decode(node.dataset.code ?? '');
    const id = `mm-${Date.now().toString(36)}-${i}`;
    try {
      const { svg } = await mermaid.render(id, code);
      const clean = DOMPurify.sanitize(svg, {
        USE_PROFILES: { svg: true, svgFilters: true }
      });
      node.innerHTML = clean;
      node.dataset.rendered = 'true';
    } catch (err) {
      node.textContent = code;
      node.dataset.rendered = 'error';
      console.warn('mermaid render failed', err);
    }
  }
}

function b64decode(s: string): string {
  try {
    return decodeURIComponent(escape(atob(s)));
  } catch {
    return '';
  }
}
