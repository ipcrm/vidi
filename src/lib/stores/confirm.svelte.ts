/**
 * Lightweight dialog store for confirming external link / image loads.
 * A single modal at the root binds to this state; any component can invoke
 * `confirmExternal({...})` and await the user's decision.
 */

export interface ConfirmRequest {
  kind: 'link' | 'image';
  url: string;
  host: string;
}

interface PendingDecision {
  approved: boolean;
  trustHost: boolean;
}

type Resolver = (d: PendingDecision) => void;

function createConfirm() {
  let request: ConfirmRequest | null = $state(null);
  let resolver: Resolver | null = null;

  function show(req: ConfirmRequest): Promise<PendingDecision> {
    request = req;
    return new Promise<PendingDecision>((resolve) => {
      resolver = resolve;
    });
  }

  function resolve(decision: PendingDecision) {
    const r = resolver;
    resolver = null;
    request = null;
    r?.(decision);
  }

  return {
    get request() {
      return request;
    },
    show,
    approve(trustHost: boolean) {
      resolve({ approved: true, trustHost });
    },
    cancel() {
      resolve({ approved: false, trustHost: false });
    }
  };
}

export const confirm = createConfirm();

/** Extract the host part of a URL, falling back to the full URL string. */
export function hostOf(url: string): string {
  try {
    return new URL(url).host;
  } catch {
    return url;
  }
}
