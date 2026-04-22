import type { Theme } from '../types';

/**
 * Theme store — reactive to prefers-color-scheme plus user override.
 * Exported as a state rune so components can bind/read.
 */
function createThemeStore() {
  let chosen: Theme = $state('system');
  let systemDark = $state(false);

  if (typeof window !== 'undefined' && window.matchMedia) {
    const mql = window.matchMedia('(prefers-color-scheme: dark)');
    systemDark = mql.matches;
    mql.addEventListener('change', (e) => {
      systemDark = e.matches;
    });
  }

  const effective = $derived(chosen === 'system' ? (systemDark ? 'dark' : 'light') : chosen);

  function apply(t: Theme) {
    chosen = t;
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.theme = effective;
    }
  }

  // Keep data-theme attribute in sync whenever effective changes.
  $effect.root(() => {
    $effect(() => {
      if (typeof document !== 'undefined') {
        document.documentElement.dataset.theme = effective;
      }
    });
  });

  return {
    get chosen() {
      return chosen;
    },
    get effective() {
      return effective;
    },
    set(t: Theme) {
      apply(t);
    }
  };
}

export const theme = createThemeStore();
