// Mirror of Rust serde types from src-tauri/src/model.rs.
// Kept hand-written (small surface, no build step).

export type Source =
  | { kind: 'localFile'; path: string }
  | { kind: 'remote'; url: string }
  | { kind: 'localFolder'; root: string };

export type Theme = 'system' | 'light' | 'dark';

export interface RenderOptions {
  theme?: Theme;
  enableMath?: boolean | null;
  enableMermaid?: boolean | null;
  baseOverride?: string | null;
}

export interface TocEntry {
  level: number;
  text: string;
  anchor: string;
}

export type ResolvedBase =
  | { kind: 'folder'; file: string; root: string }
  | { kind: 'remote'; baseUrl: string }
  | { kind: 'inline' };

export type AssetKind = 'image' | 'iframe';

export interface ExternalAsset {
  kind: AssetKind;
  url: string;
  placeholderId: string;
}

export interface RenderedDoc {
  html: string;
  title: string | null;
  toc: TocEntry[];
  wordCount: number;
  hasMath: boolean;
  hasMermaid: boolean;
  externalAssets: ExternalAsset[];
  base: ResolvedBase;
}

export interface FileNode {
  path: string;
  name: string;
  isDir: boolean;
  children: FileNode[];
}

export interface FileTree {
  root: string;
  nodes: FileNode[];
}

export interface RecentFile {
  source: Source;
  title: string;
  openedAt: number;
}

export interface Bookmark {
  id: string;
  source: Source;
  label: string;
  anchor: string | null;
  createdAt: number;
}

export interface ReadingPosition {
  scrollRatio: number;
  anchor: string | null;
  updatedAt: number;
}

export interface Settings {
  theme: Theme;
  fontScale: number;
  measureCh: number;
  trustedHosts: string[];
  confirmAlways: boolean;
  dropCaps: boolean;
  enableMath: boolean;
  enableMermaid: boolean;
}

export type LinkResolution =
  | { resolution: 'internalDoc'; source: Source }
  | { resolution: 'anchor'; fragment: string }
  | { resolution: 'external'; url: string }
  | { resolution: 'asset'; url: string; assetKind: AssetKind };

export interface RemoteFetch {
  text: string;
  finalUrl: string;
  contentType: string | null;
}

export interface SearchHit {
  path: string;
  title: string;
  score: number;
  snippet: string;
}

export interface WatchHandle {
  id: number;
}

export interface AppError {
  kind: string;
  message: string;
}

/** Source → stable string key, matches Rust `Source::key`. */
export function sourceKey(s: Source): string {
  switch (s.kind) {
    case 'localFile':
      return `file:${s.path}`;
    case 'remote':
      return `url:${s.url}`;
    case 'localFolder':
      return `folder:${s.root}`;
  }
}
