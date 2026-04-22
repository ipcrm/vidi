import { describe, it, expect } from 'vitest';
import { hostOf } from '../lib/stores/confirm.svelte';

describe('hostOf', () => {
  it('extracts host from https', () => {
    expect(hostOf('https://example.com/a/b')).toBe('example.com');
  });
  it('extracts host with port', () => {
    expect(hostOf('http://localhost:8080/x')).toBe('localhost:8080');
  });
  it('falls back to input for non-URL', () => {
    expect(hostOf('not a url')).toBe('not a url');
  });
});
