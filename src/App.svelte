<script lang="ts">
  import AddressBar from './lib/components/AddressBar.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Reader from './lib/components/Reader.svelte';
  import ConfirmDialog from './lib/components/ConfirmDialog.svelte';
  import BookmarksPanel from './lib/components/BookmarksPanel.svelte';
  import RecentsPanel from './lib/components/RecentsPanel.svelte';
  import SettingsPanel from './lib/components/SettingsPanel.svelte';
  import SearchPanel from './lib/components/SearchPanel.svelte';
  import FindBar from './lib/components/FindBar.svelte';
  import TabBar from './lib/components/TabBar.svelte';
  import HelpDialog from './lib/components/HelpDialog.svelte';
  import { ipc } from './lib/ipc';
  import { session } from './lib/stores/session.svelte';
  import { theme } from './lib/stores/theme.svelte';
  import { panels } from './lib/stores/panels.svelte';
  import {
    sourceKey,
    type FileNode,
    type Source,
    type WatchHandle
  } from './lib/types';
  import { debounce } from './lib/util/debounce';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';

  let scrollHost: HTMLElement | undefined = $state();
  let restoring = false;
  let findOpen = $state(false);
  let helpOpen = $state(false);
  let watchHandle: WatchHandle | null = null;
  let unlisten: UnlistenFn | null = null;

  // Scroll-restore dedup + suspension.
  //   - `lastRestoredKey` avoids re-restoring when the current source is the
  //     same but some other tab field changed (e.g. history index). Fixes the
  //     "TOC click needs two clicks" bug where the restore effect was
  //     overwriting the scroll-to-anchor that had just happened.
  //   - `suspendRestore` is raised while we're running an explicit navigation
  //     (back / forward), so the restore effect doesn't fight us on scroll.
  let lastRestoredKey: string | null = null;
  let suspendRestore = false;

  // Sidebar collapsed state — persisted to localStorage so it sticks.
  let sidebarCollapsed = $state(
    typeof localStorage !== 'undefined' &&
      localStorage.getItem('vidi:sidebar-collapsed') === '1'
  );
  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
    try {
      localStorage.setItem(
        'vidi:sidebar-collapsed',
        sidebarCollapsed ? '1' : '0'
      );
    } catch {
      // localStorage unavailable — in-memory toggle still works.
    }
  }

  // --- Reading position ----------------------------------------------------

  const saveScroll = debounce((source: Source, ratio: number) => {
    ipc
      .setReadingPosition(source, {
        scrollRatio: ratio,
        anchor: null,
        updatedAt: Math.floor(Date.now() / 1000)
      })
      .catch(() => {});
  }, 250);

  function onMainScroll() {
    if (restoring) return;
    const src = session.source;
    const el = scrollHost;
    if (!src || !el) return;
    const max = el.scrollHeight - el.clientHeight;
    const ratio = max > 0 ? el.scrollTop / max : 0;
    saveScroll(src, ratio);
  }

  $effect(() => {
    const src = session.source;
    const doc = session.doc;
    const el = scrollHost;
    if (!src || !doc || !el) return;

    // Svelte re-fires this effect whenever any field on the active tab
    // mutates — including history-index updates from back/forward and
    // anchor pushes from TOC clicks. We only want to restore when the
    // source truly changes, so we track the last-restored key and bail
    // out for redundant runs. Also suspended while an explicit history
    // navigation is in flight.
    const key = sourceKey(src);
    if (suspendRestore || key === lastRestoredKey) {
      lastRestoredKey = key;
      return;
    }
    lastRestoredKey = key;

    restoring = true;
    saveScroll.cancel();

    (async () => {
      try {
        const pos = await ipc.getReadingPosition(src);
        await new Promise((r) => requestAnimationFrame(() => r(null)));
        if (el && pos) {
          const max = el.scrollHeight - el.clientHeight;
          el.scrollTop = Math.max(0, pos.scrollRatio * max);
        } else if (el) {
          el.scrollTop = 0;
        }
      } finally {
        setTimeout(() => {
          restoring = false;
        }, 50);
      }
    })();
  });

  // --- Settings → live typography ------------------------------------------

  $effect(() => {
    ipc
      .getSettings()
      .then((s) => {
        theme.set(s.theme);
        document.documentElement.dataset.dropcap = s.dropCaps ? 'true' : 'false';
        document.documentElement.style.setProperty('--measure-max', `${s.measureCh}ch`);
        document.documentElement.style.setProperty(
          '--fs-body',
          `${1.0625 * s.fontScale}rem`
        );
      })
      .catch(() => {
        // First-run: theme store will stay on 'system'.
      });
  });

  // --- Folder watcher ------------------------------------------------------

  $effect(() => {
    const tree = session.tree;
    if (!tree) return;
    (async () => {
      if (watchHandle) {
        await ipc.unwatchFolder(watchHandle).catch(() => {});
        watchHandle = null;
      }
      try {
        watchHandle = await ipc.watchFolder(tree.root);
      } catch {
        // Non-fatal — watcher is a convenience.
      }
    })();

    return () => {
      if (watchHandle) {
        const h = watchHandle;
        watchHandle = null;
        ipc.unwatchFolder(h).catch(() => {});
      }
    };
  });

  // Coalesce multiple watcher events into at most one tree-refresh and one
  // active-doc re-render per burst. Mass-change events (git pull / branch
  // switch / IDE save-all) were causing hundreds of listFolder calls per
  // second, pinning the CPU.
  const refreshTreeDebounced = debounce(async () => {
    const tree = session.tree;
    if (!tree) return;
    try {
      const fresh = await ipc.listFolder(tree.root);
      session.setTree(fresh);
    } catch {
      // leave tree as-is
    }
  }, 400);

  const reRenderActiveDocDebounced = debounce(async () => {
    const src = session.source;
    if (!src || src.kind !== 'localFile') return;
    await openSource(src, { silent: true });
  }, 300);

  $effect(() => {
    (async () => {
      unlisten = await listen<{
        paths: string[];
        truncated: boolean;
        total: number;
      }>('folder://changed', (ev) => {
        const src = session.source;
        const activePath = src?.kind === 'localFile' ? src.path : null;
        const paths = ev.payload.paths;

        // If the active doc is among the changed paths, schedule a re-render.
        if (activePath && paths.includes(activePath)) {
          reRenderActiveDocDebounced();
        }

        // Any change touches the tree — schedule a refresh. The debounce
        // keeps rapid bursts (git pull, branch switch) to a single
        // `listFolder` call per ~400ms window.
        if (session.tree) {
          refreshTreeDebounced();
        }
      });
    })();

    return () => {
      refreshTreeDebounced.cancel();
      reRenderActiveDocDebounced.cancel();
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
    };
  });

  // Re-index search when the active tab's folder changes (e.g. tab switch).
  let lastIndexedRoot: string | null = null;
  $effect(() => {
    const tree = session.tree;
    if (!tree) return;
    if (tree.root === lastIndexedRoot) return;
    lastIndexedRoot = tree.root;
    ipc.indexFolder(tree.root).catch(() => {});
  });

  // --- Open flow -----------------------------------------------------------

  async function openSource(
    source: Source,
    opts?: { silent?: boolean; newTab?: boolean; anchor?: string | null }
  ) {
    if (opts?.newTab) session.newTab();
    if (!opts?.silent) session.setLoading(true);
    try {
      const doc = await ipc.renderMarkdown(source);
      session.setDoc(doc, source);
      const title = doc.title ?? describeSource(source);
      if (!opts?.silent) {
        // Record this navigation. File-watcher hot-reloads and
        // back/forward-driven navigations pass silent:true so they
        // don't pollute the history stack or the recents list.
        session.pushHistoryEntry({ source, anchor: opts?.anchor ?? null });
        ipc
          .pushRecent(source, title)
          .then(() => session.recentsChanged())
          .catch(() => {});
      }
    } catch (e) {
      session.setLoadError(formatError(e));
    } finally {
      if (!opts?.silent) session.setLoading(false);
    }
  }

  async function onOpenUrl(url: string) {
    // Address bar navigates the current tab (open a new tab first if it's
    // already holding a different document — otherwise reuse the blank tab).
    const cur = session.source;
    await openSource(
      { kind: 'remote', url },
      { newTab: !!cur }
    );
  }

  async function onOpenFolder() {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: 'Open folder'
    });
    if (typeof selected !== 'string') return;
    // Always open a folder in a new tab unless the current tab is blank.
    const cur = session.source || session.tree;
    if (cur) session.newTab();
    try {
      const tree = await ipc.listFolder(selected);
      session.setTree(tree);
      // Build search index in the background — don't block first render.
      ipc.indexFolder(tree.root).catch(() => {});
      const first = firstFile(tree.nodes);
      if (first) await openSource({ kind: 'localFile', path: first });
    } catch (e) {
      session.setLoadError(formatError(e));
    }
  }

  function newBlankTab() {
    session.newTab();
  }

  function closeActiveTab() {
    session.closeTab(session.activeId);
  }

  function firstFile(nodes: FileNode[]): string | null {
    for (const n of nodes) {
      if (n.isDir) {
        const sub = firstFile(n.children);
        if (sub) return sub;
      } else {
        return n.path;
      }
    }
    return null;
  }

  function describeSource(s: Source): string {
    switch (s.kind) {
      case 'localFile':
        return s.path.split('/').pop() ?? s.path;
      case 'remote':
        return s.url;
      case 'localFolder':
        return s.root;
    }
  }

  function formatError(e: unknown): string {
    if (typeof e === 'object' && e !== null && 'message' in e) {
      return String((e as { message: string }).message);
    }
    return String(e);
  }

  const currentPath = $derived(
    session.source && session.source.kind === 'localFile'
      ? session.source.path
      : null
  );

  function toggleTheme() {
    theme.set(theme.effective === 'dark' ? 'light' : 'dark');
  }

  /** Scroll the reader to a heading by its slug id. Used by the sidebar TOC. */
  function scrollToAnchor(anchor: string) {
    if (!scrollHost) return;
    const safe = anchor.replace(/["\\]/g, '\\$&');
    const target = scrollHost.querySelector<HTMLElement>(`[id="${safe}"]`);
    if (target) {
      target.scrollIntoView({ block: 'start', behavior: 'smooth' });
    }
  }

  /** Navigate to an anchor within the current doc, pushing a history entry. */
  function navigateToAnchor(anchor: string) {
    const src = session.source;
    if (!src) return;
    session.pushHistoryEntry({ source: src, anchor });
    scrollToAnchor(anchor);
  }

  /** Navigate to a history entry without pushing a new one. */
  async function goToEntry(source: Source, anchor: string | null) {
    const current = session.source;
    const same = current && sourceKey(current) === sourceKey(source);
    if (!same) {
      await openSource(source, { silent: true });
    }
    // Wait a frame so the new HTML is laid out before scrolling.
    await new Promise((r) => requestAnimationFrame(() => r(null)));
    if (anchor) {
      scrollToAnchor(anchor);
    } else if (scrollHost) {
      scrollHost.scrollTop = 0;
    }
  }

  async function handleBack() {
    // Raise the suspend BEFORE mutating state. session.goBack updates the
    // tab's historyIndex, which triggers the restore effect; we need the
    // flag already up by the time that effect runs (next microtask) so it
    // doesn't race scrollToAnchor / scrollTop=0.
    suspendRestore = true;
    try {
      const entry = session.goBack();
      if (entry) await goToEntry(entry.source, entry.anchor);
    } finally {
      setTimeout(() => {
        suspendRestore = false;
      }, 80);
    }
  }

  async function handleForward() {
    suspendRestore = true;
    try {
      const entry = session.goForward();
      if (entry) await goToEntry(entry.source, entry.anchor);
    } finally {
      setTimeout(() => {
        suspendRestore = false;
      }, 80);
    }
  }

  // --- Keyboard shortcuts --------------------------------------------------

  async function addBookmark() {
    const src = session.source;
    if (!src) return;
    const label = session.doc?.title ?? describeSource(src);
    try {
      await ipc.addBookmark(src, label);
      session.bookmarksChanged();
      panels.open('bookmarks');
    } catch (e) {
      session.setLoadError(formatError(e));
    }
  }

  // All command shortcuts live on the native menu (see src-tauri/src/menu.rs).
  // The menu's accelerators intercept keystrokes before the webview sees them,
  // and each click / accelerator emits a `menu://action` event with a stable
  // id that we dispatch below. The only keystroke we keep in JS is the bare
  // `?` help shortcut, since native-menu accelerators can't bind plain keys.
  function onKey(ev: KeyboardEvent) {
    if (ev.key === '?' && !ev.metaKey && !ev.ctrlKey) {
      const t = ev.target as HTMLElement | null;
      const tag = t?.tagName;
      const inField =
        tag === 'INPUT' ||
        tag === 'TEXTAREA' ||
        (t as HTMLElement | null)?.isContentEditable === true;
      if (!inField) {
        ev.preventDefault();
        helpOpen = !helpOpen;
      }
    }

    // ⌘⇥ / ⌘⇧⇥ for next / previous tab and ⌘1-9 for jump-by-index: these
    // are not in the native menu because binding Tab / digits there is
    // platform-finicky. Keep them as JS shortcuts.
    const mod = ev.metaKey || ev.ctrlKey;
    if (!mod) return;
    if (ev.key === 'Tab') {
      ev.preventDefault();
      if (ev.shiftKey) session.prevTab();
      else session.nextTab();
    } else if (/^[1-9]$/.test(ev.key)) {
      ev.preventDefault();
      session.activateByIndex(parseInt(ev.key, 10) - 1);
    }
  }

  // --- Menu event dispatch -------------------------------------------------
  async function dispatchMenuAction(id: string) {
    switch (id) {
      case 'open_file':
        await onOpenFile();
        break;
      case 'open_folder':
        await onOpenFolder();
        break;
      case 'open_recents':
      case 'panel_recents':
        panels.toggle('recents');
        break;
      case 'new_tab':
        newBlankTab();
        break;
      case 'close_tab':
        closeActiveTab();
        break;
      case 'bookmark':
        await addBookmark();
        break;
      case 'print':
        ipc.printPage().catch((err) => console.warn('print failed:', err));
        break;
      case 'toggle_sidebar':
        toggleSidebar();
        break;
      case 'panel_bookmarks':
        panels.toggle('bookmarks');
        break;
      case 'panel_settings':
        panels.toggle('settings');
        break;
      case 'search_folder':
        panels.toggle('search');
        break;
      case 'find_in_doc':
        findOpen = true;
        break;
      case 'toggle_theme':
        toggleTheme();
        break;
      case 'help_shortcuts':
        helpOpen = !helpOpen;
        break;
      case 'help_docs':
        await ipc
          .openExternal('https://ipcrm.github.io/vidi')
          .catch(() => {});
        break;
      case 'go_back':
        await handleBack();
        break;
      case 'go_forward':
        await handleForward();
        break;
    }
  }

  async function onOpenFile() {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: 'Markdown', extensions: ['md', 'markdown'] }]
    });
    if (typeof selected !== 'string') return;
    const cur = session.source;
    await openSource(
      { kind: 'localFile', path: selected },
      { newTab: !!cur }
    );
  }

  // Open .md files handed to us by the OS (macOS "Open With", drag-to-dock).
  async function openFilesFromOS(paths: string[]) {
    for (const path of paths) {
      if (!/\.(md|markdown)$/i.test(path)) continue;
      const cur = session.source;
      await openSource(
        { kind: 'localFile', path },
        { newTab: !!cur }
      );
    }
  }

  let unlistenMenu: UnlistenFn | null = null;
  let unlistenOpen: UnlistenFn | null = null;
  $effect(() => {
    (async () => {
      unlistenMenu = await listen<string>('menu://action', (ev) => {
        dispatchMenuAction(ev.payload).catch((err) =>
          console.warn('menu action failed:', ev.payload, err)
        );
      });

      // Subsequent OS-opens while the app is already running.
      unlistenOpen = await listen<string[]>('vidi://open-paths', (ev) => {
        openFilesFromOS(ev.payload).catch((err) =>
          console.warn('open paths failed:', err)
        );
      });

      // Drain any paths that landed before the webview was ready (cold
      // launch via "Open With" on macOS).
      try {
        const pending = await ipc.takePendingOpens();
        if (pending.length > 0) await openFilesFromOS(pending);
      } catch (err) {
        console.warn('drain pending opens failed:', err);
      }
    })();
    return () => {
      if (unlistenMenu) {
        unlistenMenu();
        unlistenMenu = null;
      }
      if (unlistenOpen) {
        unlistenOpen();
        unlistenOpen = null;
      }
    };
  });
</script>

<svelte:window onkeydown={onKey} />

<div class="app">
  {#if session.tabs.length > 1}
    <TabBar onNewTab={newBlankTab} />
  {/if}

  <AddressBar
    title={session.doc?.title ?? null}
    busy={session.loading}
    canGoBack={session.canGoBack}
    canGoForward={session.canGoForward}
    onBack={handleBack}
    onForward={handleForward}
    {onOpenUrl}
    {onOpenFolder}
  />

  <div class="body">
    <Sidebar
      tree={session.tree}
      toc={session.doc?.toc ?? []}
      docTitle={session.doc?.title ?? null}
      current={currentPath}
      collapsed={sidebarCollapsed}
      onOpen={(s) => openSource(s)}
      onTocClick={navigateToAnchor}
      onToggle={toggleSidebar}
    />

    <main class="main" bind:this={scrollHost} onscroll={onMainScroll}>
      {#if session.loadError}
        <div class="state error">
          <h3>Couldn't render that</h3>
          <p>{session.loadError}</p>
        </div>
      {:else if session.loading}
        <div class="state loading">Loading…</div>
      {:else if session.doc}
        <Reader
          doc={session.doc}
          onNavigate={(s) => openSource(s)}
          onAnchorNavigate={navigateToAnchor}
        />
      {:else}
        <div class="state empty">
          <h2 class="empty-title">Vidi</h2>
          <p class="empty-sub">
            Open a folder of markdown files, or paste a GitHub URL above.
          </p>

          <dl class="shortcut-list" aria-label="Keyboard shortcuts">
            <dt>Open</dt>
            <dd><kbd>⌘O</kbd> open folder</dd>

            <dt>Tabs</dt>
            <dd>
              <kbd>⌘T</kbd> new tab ·
              <kbd>⌘W</kbd> close tab ·
              <kbd>⌘⇥</kbd> / <kbd>⌘⇧⇥</kbd> next / previous ·
              <kbd>⌘1</kbd>…<kbd>⌘9</kbd> jump to tab
            </dd>

            <dt>Navigation</dt>
            <dd>
              <kbd>⌘[</kbd> back ·
              <kbd>⌘]</kbd> forward
            </dd>

            <dt>Layout</dt>
            <dd>
              <kbd>⌘\</kbd> toggle sidebar
            </dd>

            <dt>Search</dt>
            <dd>
              <kbd>⌘F</kbd> find in document ·
              <kbd>⌘⇧F</kbd> search across folder
            </dd>

            <dt>Bookmarks</dt>
            <dd>
              <kbd>⌘D</kbd> bookmark current doc ·
              <kbd>⌘B</kbd> bookmarks panel ·
              <kbd>⌘Y</kbd> recents panel
            </dd>

            <dt>Settings</dt>
            <dd>
              <kbd>⌘,</kbd> open settings
            </dd>

            <dt>Print</dt>
            <dd>
              <kbd>⌘P</kbd> print / save as PDF
            </dd>

            <dt>Help</dt>
            <dd>
              <kbd>?</kbd> or <kbd>⌘/</kbd> show this list anywhere
            </dd>
          </dl>
        </div>
      {/if}
    </main>

    {#if panels.active}
      <aside class="side-panel">
        {#if panels.active === 'bookmarks'}
          <BookmarksPanel onOpen={(s) => openSource(s)} />
        {:else if panels.active === 'recents'}
          <RecentsPanel onOpen={(s) => openSource(s)} />
        {:else if panels.active === 'settings'}
          <SettingsPanel />
        {:else if panels.active === 'search'}
          <SearchPanel onOpen={(s) => openSource(s)} />
        {/if}
      </aside>
    {/if}
  </div>

  <FindBar container={scrollHost} open={findOpen} onClose={() => (findOpen = false)} />

  <button
    type="button"
    class="theme-toggle"
    onclick={toggleTheme}
    aria-label={theme.effective === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
    title={theme.effective === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
  >
    {#if theme.effective === 'dark'}
      <!-- Sun -->
      <svg
        viewBox="0 0 24 24"
        width="16"
        height="16"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="4" fill="currentColor" stroke="none" />
        <line x1="12" y1="2.5" x2="12" y2="5" />
        <line x1="12" y1="19" x2="12" y2="21.5" />
        <line x1="2.5" y1="12" x2="5" y2="12" />
        <line x1="19" y1="12" x2="21.5" y2="12" />
        <line x1="4.9" y1="4.9" x2="6.7" y2="6.7" />
        <line x1="17.3" y1="17.3" x2="19.1" y2="19.1" />
        <line x1="4.9" y1="19.1" x2="6.7" y2="17.3" />
        <line x1="17.3" y1="6.7" x2="19.1" y2="4.9" />
      </svg>
    {:else}
      <!-- Crescent moon (thick, readable) -->
      <svg
        viewBox="0 0 24 24"
        width="16"
        height="16"
        fill="currentColor"
        aria-hidden="true"
      >
        <path d="M20 14.5A8.5 8.5 0 1 1 9.5 4a6.75 6.75 0 0 0 10.5 10.5Z" />
      </svg>
    {/if}
  </button>

  <ConfirmDialog />

  <HelpDialog open={helpOpen} onClose={() => (helpOpen = false)} />
</div>

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    background: var(--paper);
    color: var(--ink);
  }
  :global(body) {
    font-family: 'Source Serif 4', 'Source Serif Pro', 'Iowan Old Style',
      'Palatino', 'Georgia', ui-serif, serif;
    font-size: var(--fs-body);
    line-height: var(--leading);
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .body {
    display: flex;
    flex: 1 1 auto;
    min-height: 0;
  }

  .main {
    flex: 1 1 auto;
    overflow: auto;
    position: relative;
  }

  .side-panel {
    width: 320px;
    min-width: 240px;
    max-width: 480px;
    border-left: 1px solid var(--rule);
    background: var(--chrome-bg);
    resize: horizontal;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .state {
    padding: 4rem 2rem;
    max-width: 640px;
    margin: 0 auto;
    font-family: var(--font-ui, system-ui);
    color: var(--ink-dim);
  }
  .state.error {
    color: var(--danger, #b42b1a);
  }
  .state h3 {
    margin-top: 0;
  }
  .empty-title {
    font-family: 'Source Serif 4', Georgia, serif;
    font-size: 2rem;
    font-weight: 400;
    letter-spacing: 0.01em;
    color: var(--ink);
    margin-bottom: 0.25rem;
  }
  .empty-sub {
    color: var(--ink-dim);
    margin-bottom: 2rem;
  }

  .shortcut-list {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.45rem 1.25rem;
    margin: 0;
    padding: 1rem 0 0;
    border-top: 1px solid var(--rule);
    font-size: 0.85rem;
    line-height: 1.55;
    color: var(--ink);
  }
  .shortcut-list dt {
    font-family: var(--font-ui, system-ui);
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-dim);
    align-self: center;
  }
  .shortcut-list dd {
    margin: 0;
    color: var(--ink-dim);
  }
  .shortcut-list kbd {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.72rem;
    padding: 0.12rem 0.4rem;
    border: 1px solid var(--rule);
    border-radius: 3px;
    background: var(--paper);
    color: var(--ink);
    margin-right: 0.1rem;
    white-space: nowrap;
  }

  .theme-toggle {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 1px solid var(--rule);
    background: var(--paper);
    color: var(--ink);
    cursor: pointer;
    display: grid;
    place-items: center;
    z-index: 20;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
    transition: background 120ms, color 120ms, transform 120ms;
  }
  .theme-toggle:hover {
    background: var(--chrome-hover);
    color: var(--accent);
  }
  .theme-toggle:active {
    transform: scale(0.95);
  }
  .theme-toggle:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
