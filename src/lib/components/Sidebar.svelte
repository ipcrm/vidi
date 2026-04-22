<script lang="ts">
  import type { FileTree, FileNode, Source, TocEntry } from '../types';

  interface Props {
    tree: FileTree | null;
    toc: TocEntry[];
    current: string | null;
    docTitle: string | null;
    collapsed: boolean;
    onOpen: (source: Source) => void;
    onTocClick: (anchor: string) => void;
    onToggle: () => void;
  }

  const {
    tree,
    toc,
    current,
    docTitle,
    collapsed,
    onOpen,
    onTocClick,
    onToggle
  }: Props = $props();

  let filter = $state('');

  // Remember each section's expanded state across reopens.
  const loadFlag = (key: string, def: boolean): boolean => {
    if (typeof localStorage === 'undefined') return def;
    const raw = localStorage.getItem(key);
    return raw === null ? def : raw === '1';
  };
  const saveFlag = (key: string, v: boolean) => {
    try {
      localStorage.setItem(key, v ? '1' : '0');
    } catch {
      // ignore
    }
  };

  let filesOpen = $state(loadFlag('visum:sidebar:files', true));
  let tocOpen = $state(loadFlag('visum:sidebar:toc', true));

  $effect(() => saveFlag('visum:sidebar:files', filesOpen));
  $effect(() => saveFlag('visum:sidebar:toc', tocOpen));

  function rootLabel(path: string): string {
    const parts = path.split('/').filter(Boolean);
    return parts[parts.length - 1] ?? path;
  }

  const matches = (n: FileNode): boolean => {
    if (!filter) return true;
    const q = filter.toLowerCase();
    if (n.name.toLowerCase().includes(q)) return true;
    if (n.isDir) return n.children.some(matches);
    return false;
  };

  function open(path: string) {
    onOpen({ kind: 'localFile', path });
  }

  function onKeydown(ev: KeyboardEvent, path: string) {
    if (ev.key === 'Enter') {
      ev.preventDefault();
      open(path);
    }
  }
</script>

{#if collapsed}
  <div class="sidebar-rail" aria-label="File tree (collapsed)">
    <button
      type="button"
      class="rail-btn"
      onclick={onToggle}
      aria-label="Show sidebar"
      title="Show sidebar (⌘\\)"
    >
      ›
    </button>
  </div>
{:else}
  <aside class="sidebar" aria-label="Navigation">
    <header class="sidebar-head">
      <h2 class="sidebar-title">
        {tree ? rootLabel(tree.root) : docTitle ?? 'Visum'}
      </h2>
      <button
        type="button"
        class="collapse-btn"
        onclick={onToggle}
        aria-label="Hide sidebar"
        title="Hide sidebar (⌘\\)"
      >
        ‹
      </button>
    </header>

    <!-- Files section --------------------------------------------------- -->
    <section class="section" class:open={filesOpen}>
      <button
        type="button"
        class="section-summary"
        aria-expanded={filesOpen}
        onclick={() => (filesOpen = !filesOpen)}
      >
        <span class="section-caret" aria-hidden="true"></span>
        <span class="section-title">Files</span>
      </button>
      {#if filesOpen}
        <div class="section-body">
          {#if tree}
            <input
              class="filter"
              type="search"
              placeholder="Filter files…"
              bind:value={filter}
              aria-label="Filter files"
            />
            <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
            <nav class="tree" role="tree" aria-label="Files">
              {#each tree.nodes.filter(matches) as node (node.path)}
                {@render renderNode(node, 0)}
              {/each}
            </nav>
          {:else}
            <p class="section-empty">No folder open — ⌘O to open one.</p>
          {/if}
        </div>
      {/if}
    </section>

    <!-- Table of contents section --------------------------------------- -->
    <section class="section" class:open={tocOpen}>
      <button
        type="button"
        class="section-summary"
        aria-expanded={tocOpen}
        onclick={() => (tocOpen = !tocOpen)}
      >
        <span class="section-caret" aria-hidden="true"></span>
        <span class="section-title">Contents</span>
        {#if toc.length > 0}
          <span class="section-count">{toc.length}</span>
        {/if}
      </button>
      {#if tocOpen}
        <div class="section-body">
          {#if toc.length === 0}
            <p class="section-empty">
              {docTitle ? 'No headings in this doc.' : 'Open a doc to see its contents.'}
            </p>
          {:else}
            <nav class="toc" aria-label="Table of contents">
              {#each toc as entry (entry.anchor + entry.level)}
                <button
                  type="button"
                  class="toc-item toc-level-{entry.level}"
                  onclick={() => onTocClick(entry.anchor)}
                  title={entry.text}
                >
                  {entry.text}
                </button>
              {/each}
            </nav>
          {/if}
        </div>
      {/if}
    </section>
  </aside>
{/if}

{#snippet renderNode(node: FileNode, depth: number)}
  {#if node.isDir}
    <details class="tree-dir" open>
      <summary style="--depth: {depth}">
        <span class="caret" aria-hidden="true"></span>
        <span class="dirname">{node.name}</span>
      </summary>
      {#each node.children.filter(matches) as child (child.path)}
        {@render renderNode(child, depth + 1)}
      {/each}
    </details>
  {:else}
    <button
      type="button"
      class="tree-file"
      class:active={current === node.path}
      style="--depth: {depth}"
      onclick={() => open(node.path)}
      onkeydown={(e) => onKeydown(e, node.path)}
      role="treeitem"
      aria-selected={current === node.path}
    >
      <span class="filename">{node.name}</span>
    </button>
  {/if}
{/snippet}

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 280px;
    min-width: 200px;
    max-width: 520px;
    resize: horizontal;
    overflow: hidden;
    border-right: 1px solid var(--rule);
    background: var(--chrome-bg);
    font-family: var(--font-ui, system-ui);
    font-size: 0.875rem;
    flex: 0 0 auto;
    height: 100%;
  }

  .sidebar-rail {
    flex: 0 0 auto;
    width: 24px;
    border-right: 1px solid var(--rule);
    background: var(--chrome-bg);
    display: flex;
    justify-content: center;
    padding-top: 0.5rem;
  }
  .rail-btn {
    width: 20px;
    height: 28px;
    background: var(--paper);
    border: 1px solid var(--rule);
    border-radius: 3px;
    color: var(--ink-dim);
    cursor: pointer;
    font-size: 0.875rem;
    line-height: 1;
    padding: 0;
    display: grid;
    place-items: center;
  }
  .rail-btn:hover {
    background: var(--chrome-hover);
    color: var(--ink);
  }

  .sidebar-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.375rem;
    padding: 0.75rem 0.875rem 0.625rem;
    border-bottom: 1px solid var(--rule);
    background: var(--chrome-bg);
    flex: 0 0 auto;
  }

  .sidebar-title {
    font-family: var(--font-serif);
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--ink);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1 1 auto;
    min-width: 0;
  }

  .collapse-btn {
    background: none;
    border: 0;
    color: var(--ink-dim);
    cursor: pointer;
    padding: 0 0.375rem;
    border-radius: 3px;
    font-size: 0.9rem;
    line-height: 1.2;
    flex: 0 0 auto;
  }
  .collapse-btn:hover {
    background: var(--chrome-hover);
    color: var(--ink);
  }

  /* Sections — collapsible groups sharing the sidebar column.
     Each open section takes an equal share of the remaining vertical space;
     a collapsed section shrinks to just its summary row. When only one is
     open it naturally expands to fill all available space. */
  .section {
    display: flex;
    flex-direction: column;
    border-bottom: 1px solid var(--rule);
    min-height: 0;
    flex: 0 0 auto;
  }
  .section.open {
    flex: 1 1 0;
    min-height: 0;
  }
  .section-summary {
    width: 100%;
    text-align: left;
    background: none;
    border: 0;
    cursor: pointer;
    user-select: none;
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.875rem;
    color: var(--ink-dim);
    font: inherit;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    flex: 0 0 auto;
  }
  .section-summary:hover {
    background: var(--chrome-hover);
    color: var(--ink);
  }
  .section-summary:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .section-body {
    flex: 1 1 auto;
    overflow-y: auto;
    min-height: 0;
  }
  .section-caret {
    display: inline-block;
    width: 0;
    height: 0;
    border-left: 4px solid currentColor;
    border-top: 4px solid transparent;
    border-bottom: 4px solid transparent;
    transition: transform 120ms;
    flex: 0 0 auto;
  }
  .section.open .section-caret {
    transform: rotate(90deg);
  }
  .section-title {
    flex: 1 1 auto;
  }
  .section-count {
    font-size: 0.625rem;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: var(--ink-dim);
    padding: 0.05rem 0.35rem;
    background: var(--paper);
    border-radius: 8px;
    letter-spacing: 0;
  }
  .section-empty {
    padding: 0.75rem 0.875rem 1rem;
    color: var(--ink-dim);
    font-size: 0.8125rem;
    margin: 0;
  }

  .filter {
    width: calc(100% - 1.25rem);
    margin: 0.25rem 0.625rem 0.5rem;
    padding: 0.375rem 0.5rem;
    border: 1px solid var(--rule);
    border-radius: 4px;
    background: var(--paper);
    color: var(--ink);
    font: inherit;
    display: block;
  }

  .tree {
    padding: 0 0.25rem 0.75rem;
  }

  /* Table of contents ------------------------------------------------- */
  .toc {
    display: flex;
    flex-direction: column;
    padding: 0.25rem 0.25rem 0.75rem;
  }
  .toc-item {
    text-align: left;
    background: none;
    border: 0;
    color: var(--ink);
    padding: 0.2rem 0.625rem;
    padding-left: calc(0.625rem + (var(--toc-indent, 0) * 0.75rem));
    cursor: pointer;
    font: inherit;
    font-size: 0.8125rem;
    line-height: 1.35;
    border-radius: 3px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .toc-item:hover {
    background: var(--chrome-hover);
    color: var(--accent);
  }
  .toc-item:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .toc-level-1 {
    --toc-indent: 0;
    font-weight: 600;
    color: var(--ink);
  }
  .toc-level-2 {
    --toc-indent: 1;
  }
  .toc-level-3 {
    --toc-indent: 2;
    color: var(--ink-dim);
  }
  .toc-level-4 {
    --toc-indent: 3;
    color: var(--ink-dim);
    font-size: 0.78rem;
  }
  .toc-level-5,
  .toc-level-6 {
    --toc-indent: 4;
    color: var(--ink-dim);
    font-size: 0.75rem;
  }

  .tree-dir {
    margin: 0;
  }

  .tree-dir > summary {
    list-style: none;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    padding-left: calc(0.5rem + var(--depth, 0) * 0.875rem);
    user-select: none;
    color: var(--ink-dim);
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .tree-dir > summary::-webkit-details-marker {
    display: none;
  }

  .caret {
    display: inline-block;
    width: 0;
    height: 0;
    border-left: 4px solid currentColor;
    border-top: 4px solid transparent;
    border-bottom: 4px solid transparent;
    transform: rotate(0deg);
    transition: transform 120ms;
  }
  .tree-dir[open] > summary .caret {
    transform: rotate(90deg);
  }

  .tree-file {
    display: block;
    width: 100%;
    text-align: left;
    padding: 0.25rem 0.5rem;
    padding-left: calc(0.5rem + var(--depth, 0) * 0.875rem + 0.875rem);
    background: none;
    border: 0;
    color: var(--ink);
    font: inherit;
    cursor: pointer;
    border-radius: 3px;
  }
  .tree-file:hover {
    background: var(--chrome-hover);
  }
  .tree-file.active {
    background: var(--accent-bg);
    color: var(--accent);
  }
  .tree-file:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

</style>
