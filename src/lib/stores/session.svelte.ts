import { sourceKey, type FileTree, type RenderedDoc, type Source } from '../types';

/**
 * Multi-tab session with per-tab back/forward history.
 *
 * Each tab holds its own document, folder tree, loading/error state, and a
 * browser-style navigation stack. Trusted hosts stay session-wide.
 */

/** One entry in a tab's navigation history. */
export interface HistoryEntry {
  source: Source;
  anchor: string | null;
}

export interface Tab {
  id: string;
  doc: RenderedDoc | null;
  source: Source | null;
  tree: FileTree | null;
  loading: boolean;
  loadError: string | null;
  history: HistoryEntry[];
  /** Index into `history` for the currently-displayed entry, or -1 if empty. */
  historyIndex: number;
}

const MAX_HISTORY = 100;

function newBlankTab(): Tab {
  return {
    id: crypto.randomUUID(),
    doc: null,
    source: null,
    tree: null,
    loading: false,
    loadError: null,
    history: [],
    historyIndex: -1
  };
}

function createSession() {
  const firstTab = newBlankTab();
  let tabs: Tab[] = $state([firstTab]);
  let activeId: string = $state(firstTab.id);
  let trustedHosts: Set<string> = $state(new Set());

  // Monotonic "something changed" counters used by panels to decide when to
  // re-fetch from disk. Cheaper than a full pub-sub for a handful of stores.
  let bookmarksVersion = $state(0);
  let recentsVersion = $state(0);

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

    // --- Change signals -------------------------------------------------
    get bookmarksVersion() {
      return bookmarksVersion;
    },
    get recentsVersion() {
      return recentsVersion;
    },
    bookmarksChanged() {
      bookmarksVersion += 1;
    },
    recentsChanged() {
      recentsVersion += 1;
    },

    // --- Back / forward history ---------------------------------------
    get canGoBack() {
      return findActive().historyIndex > 0;
    },
    get canGoForward() {
      const t = findActive();
      return t.historyIndex < t.history.length - 1;
    },
    /**
     * Push an entry onto the active tab's history. Truncates any
     * forward entries (browser-style: navigating from the middle of the
     * stack drops the forward tail).
     *
     * If the incoming entry is identical to the current head (same source
     * + same anchor), this is a no-op — keeps the stack tidy across
     * rapid re-renders of the same thing.
     */
    pushHistoryEntry(entry: HistoryEntry) {
      const i = tabs.findIndex((t) => t.id === activeId);
      if (i === -1) return;
      const tab = tabs[i];
      const current = tab.history[tab.historyIndex];
      if (
        current &&
        sourceKey(current.source) === sourceKey(entry.source) &&
        (current.anchor ?? null) === (entry.anchor ?? null)
      ) {
        return;
      }
      const truncated = tab.history.slice(0, tab.historyIndex + 1);
      truncated.push(entry);
      const capped =
        truncated.length > MAX_HISTORY
          ? truncated.slice(truncated.length - MAX_HISTORY)
          : truncated;
      tabs[i] = {
        ...tab,
        history: capped,
        historyIndex: capped.length - 1
      };
    },
    /** Step the history cursor backward; returns the entry to navigate to. */
    goBack(): HistoryEntry | null {
      const i = tabs.findIndex((t) => t.id === activeId);
      if (i === -1) return null;
      const tab = tabs[i];
      if (tab.historyIndex <= 0) return null;
      const next = tab.history[tab.historyIndex - 1];
      tabs[i] = { ...tab, historyIndex: tab.historyIndex - 1 };
      return next;
    },
    /** Step the history cursor forward; returns the entry to navigate to. */
    goForward(): HistoryEntry | null {
      const i = tabs.findIndex((t) => t.id === activeId);
      if (i === -1) return null;
      const tab = tabs[i];
      if (tab.historyIndex >= tab.history.length - 1) return null;
      const next = tab.history[tab.historyIndex + 1];
      tabs[i] = { ...tab, historyIndex: tab.historyIndex + 1 };
      return next;
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
