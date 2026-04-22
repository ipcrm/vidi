export type PanelName = 'bookmarks' | 'recents' | 'settings' | 'search';

function createPanels() {
  let active: PanelName | null = $state(null);

  return {
    get active() {
      return active;
    },
    open(name: PanelName) {
      active = name;
    },
    close() {
      active = null;
    },
    toggle(name: PanelName) {
      active = active === name ? null : name;
    }
  };
}

export const panels = createPanels();
