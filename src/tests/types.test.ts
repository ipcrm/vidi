import { describe, it, expect } from 'vitest';
import { sourceKey } from '../lib/types';

describe('sourceKey', () => {
  it('formats local file', () => {
    expect(sourceKey({ kind: 'localFile', path: '/tmp/a.md' })).toBe('file:/tmp/a.md');
  });
  it('formats remote', () => {
    expect(sourceKey({ kind: 'remote', url: 'https://x' })).toBe('url:https://x');
  });
  it('formats folder', () => {
    expect(sourceKey({ kind: 'localFolder', root: '/docs' })).toBe('folder:/docs');
  });
});
