/**
 * Trailing-edge debounce. Invokes `fn` once `ms` milliseconds have passed
 * since the last call. The returned function also exposes `.cancel()` to
 * drop a pending invocation (e.g. on component unmount).
 */
export function debounce<A extends unknown[]>(
  fn: (...args: A) => void,
  ms: number
): ((...args: A) => void) & { cancel: () => void } {
  let handle: ReturnType<typeof setTimeout> | null = null;
  const wrapped = (...args: A) => {
    if (handle !== null) clearTimeout(handle);
    handle = setTimeout(() => {
      handle = null;
      fn(...args);
    }, ms);
  };
  wrapped.cancel = () => {
    if (handle !== null) {
      clearTimeout(handle);
      handle = null;
    }
  };
  return wrapped;
}
