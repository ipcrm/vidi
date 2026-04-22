import { describe, it, expect, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => invokeMock(cmd, args)
}));

import { ipc } from '../lib/ipc';

describe('ipc wrappers', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('renderMarkdown passes source + options', async () => {
    await ipc.renderMarkdown({ kind: 'remote', url: 'https://x' });
    expect(invokeMock).toHaveBeenCalledWith('render_markdown', {
      source: { kind: 'remote', url: 'https://x' },
      options: undefined
    });
  });

  it('renderMarkdownInline passes text + baseUrl', async () => {
    await ipc.renderMarkdownInline('# hi', 'https://base');
    expect(invokeMock).toHaveBeenCalledWith('render_markdown_inline', {
      text: '# hi',
      baseUrl: 'https://base'
    });
  });

  it('listFolder sends path arg', async () => {
    await ipc.listFolder('/docs');
    expect(invokeMock).toHaveBeenCalledWith('list_folder', { path: '/docs' });
  });

  it('pushRecent sends source + title', async () => {
    const src = { kind: 'localFile', path: '/x.md' } as const;
    await ipc.pushRecent(src, 'X');
    expect(invokeMock).toHaveBeenCalledWith('push_recent', {
      source: src,
      title: 'X'
    });
  });

  it('openExternal sends url arg', async () => {
    await ipc.openExternal('https://x');
    expect(invokeMock).toHaveBeenCalledWith('open_external', { url: 'https://x' });
  });

  it('listRecents has no args', async () => {
    await ipc.listRecents();
    expect(invokeMock).toHaveBeenCalledWith('list_recents', undefined);
  });
});
