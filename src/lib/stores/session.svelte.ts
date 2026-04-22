import type { FileTree, RenderedDoc, Source } from '../types';

/**
 * Multi-tab session.
 *
 * Each tab holds its own document, folder tree, loading/error state. Trusted
 * hosts and the "session" API surface (doc/source/tree/loading/loadError/
 * setDoc/setLoading/setLoadError/setTree) delegate to the active tab so the
 * rest of the app can stay tab-agnostic.
 */

export interface Tab {
  id: string;
  doc: RenderedDoc | null;
  source: Source | null;
  tree: FileTree | null;
  loading: boolean;
  loadError: string | null;
}

function newBlankTab(): Tab {
  return {
    id: crypto.randomUUID(),
    doc: null,
    source: null,
    tree: null,
    loading: false,
    loadError: null
  };
}

function createSession() {
  let tabs: Tab[] = $state([newBlankTab()]);
  let activeId: string = $state(tabs[0].id);
  let trustedHosts: Set<string> = $state(new Set());

  function findActive(): Tab {
    return tabs.find((t) => t.id === activeId) ?? tabs[0];
  }

  function updateActive(patch: Partial<Tab>) {
    const i = tabs.findIndex((t) => t.id === activeId);
    if (i === -1) return;
    tabs[i] = { ...tabs[i], ...patch };
  }

  function titleForTab(t: Tab): string {
    if (t.doc?.title) return t.doc.title;
    if (t.source) {
      switch (t.source.kind) {
        case 'localFile':
          return t.source.path.split('/').pop() ?? t.source.path;
        case 'remote':
          return t.source.url;
        case 'localFolder':
          return t.source.root.split('/').pop() ?? t.source.root;
      }
    }
    if (t.tree) return t.tree.root.split('/').pop() ?? t.tree.root;
    return 'New tab';
  }

  return {
    // --- Active-tab view (backwards-compatible API) --------------------
    get doc() {
      return findActive().doc;
    },
    get source() {
      return findActive().source;
    },
    get loading() {
      return findActive().loading;
    },
    get loadError() {
      return findActive().loadError;
    },
    get tree() {
      return findActive().tree;
    },
    setDoc(d: RenderedDoc | null, s: Source | null) {
      updateActive({ doc: d, source: s, loadError: null });
    },
    setLoading(v: boolean) {
      updateActive({ loading: v, loadError: v ? null : findActive().loadError });
    },
    setLoadError(msg: string | null) {
      updateActive({ loadError: msg });
    },
    setTree(t: FileTree | null) {
      updateActive({ tree: t });
    },

    // --- Trusted hosts (session-wide) ----------------------------------
    get trustedHosts() {
      return trustedHosts;
    },
    trustHost(host: string) {
      trustedHosts = new Set([...trustedHosts, host]);
    },
    isHostTrusted(host: string): boolean {
      return trustedHosts.has(host);
    },

    // --- Tab management ------------------------------------------------
    get tabs() {
      return tabs;
    },
    get activeId() {
      return activeId;
    },
    get activeIndex() {
      return tabs.findIndex((t) => t.id === activeId);
    },
    titleForTab,
    titleForActive(): string {
      return titleForTab(findActive());
    },
    activate(id: string) {
      if (tabs.some((t) => t.id === id)) activeId = id;
    },
    newTab(): string {
      const t = newBlankTab();
      const idx = tabs.findIndex((x) => x.id === activeId);
      if (idx >= 0) {
        tabs = [...tabs.slice(0, idx + 1), t, ...tabs.slice(idx + 1)];
      } else {
        tabs = [...tabs, t];
      }
      activeId = t.id;
      return t.id;
    },
    /** Close a tab by id. Returns the id of the tab now active. */
    closeTab(id: string): string {
      const i = tabs.findIndex((t) => t.id === id);
      if (i === -1) return activeId;
      // Never leave zero tabs — replace with a blank if this was the last.
      if (tabs.length === 1) {
        tabs = [newBlankTab()];
        activeId = tabs[0].id;
        return activeId;
      }
      const wasActive = id === activeId;
      tabs = tabs.filter((t) => t.id !== id);
      if (wasActive) {
        const next = tabs[Math.min(i, tabs.length - 1)];
        activeId = next.id;
      }
      return activeId;
    },
    nextTab() {
      if (tabs.length < 2) return;
      const i = tabs.findIndex((t) => t.id === activeId);
      activeId = tabs[(i + 1) % tabs.length].id;
    },
    prevTab() {
      if (tabs.length < 2) return;
      const i = tabs.findIndex((t) => t.id === activeId);
      activeId = tabs[(i - 1 + tabs.length) % tabs.length].id;
    },
    activateByIndex(idx: number) {
      if (idx >= 0 && idx < tabs.length) activeId = tabs[idx].id;
    }
  };
}

export const session = createSession();
