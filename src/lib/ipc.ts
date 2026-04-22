import { invoke } from '@tauri-apps/api/core';
import type {
  Bookmark,
  FileTree,
  LinkResolution,
  ReadingPosition,
  RecentFile,
  RemoteFetch,
  RenderOptions,
  RenderedDoc,
  ResolvedBase,
  SearchHit,
  Settings,
  Source,
  WatchHandle
} from './types';

/**
 * Typed wrappers over `invoke()`. Centralises command names + argument
 * shapes so components never call `invoke` directly.
 */
export const ipc = {
  renderMarkdown(source: Source, options?: RenderOptions): Promise<RenderedDoc> {
    return invoke('render_markdown', { source, options });
  },
  renderMarkdownInline(text: string, baseUrl?: string): Promise<RenderedDoc> {
    return invoke('render_markdown_inline', { text, baseUrl });
  },
  listFolder(path: string): Promise<FileTree> {
    return invoke('list_folder', { path });
  },
  readFile(path: string): Promise<string> {
    return invoke('read_file', { path });
  },
  fetchRemote(url: string): Promise<RemoteFetch> {
    return invoke('fetch_remote', { url });
  },
  resolveLink(href: string, base: ResolvedBase): Promise<LinkResolution> {
    return invoke('resolve_link', { href, base });
  },
  watchFolder(path: string): Promise<WatchHandle> {
    return invoke('watch_folder', { path });
  },
  unwatchFolder(handle: WatchHandle): Promise<void> {
    return invoke('unwatch_folder', { handle });
  },
  listRecents(): Promise<RecentFile[]> {
    return invoke('list_recents');
  },
  pushRecent(source: Source, title: string): Promise<void> {
    return invoke('push_recent', { source, title });
  },
  listBookmarks(): Promise<Bookmark[]> {
    return invoke('list_bookmarks');
  },
  addBookmark(source: Source, label: string, anchor?: string | null): Promise<Bookmark> {
    return invoke('add_bookmark', { source, label, anchor });
  },
  removeBookmark(id: string): Promise<void> {
    return invoke('remove_bookmark', { id });
  },
  getReadingPosition(source: Source): Promise<ReadingPosition | null> {
    return invoke('get_reading_position', { source });
  },
  setReadingPosition(source: Source, position: ReadingPosition): Promise<void> {
    return invoke('set_reading_position', { source, position });
  },
  getSettings(): Promise<Settings> {
    return invoke('get_settings');
  },
  setSettings(settings: Settings): Promise<void> {
    return invoke('set_settings', { settings });
  },
  openExternal(url: string): Promise<void> {
    return invoke('open_external', { url });
  },
  indexFolder(path: string): Promise<number> {
    return invoke('index_folder', { path });
  },
  searchFolder(query: string, limit?: number): Promise<SearchHit[]> {
    return invoke('search_folder', { query, limit });
  },
  clearIndex(): Promise<void> {
    return invoke('clear_index');
  }
};
