export type Theme = 'system' | 'light' | 'dark';
const KEY = 'openlofree-theme';
const order: Theme[] = ['system', 'light', 'dark'];

function read(): Theme {
  try {
    const v = localStorage.getItem(KEY);
    if (v === 'light' || v === 'dark') return v;
  } catch {
    // Storage can be blocked, the app works without it.
  }
  return 'system';
}

function apply(theme: Theme) {
  if (theme === 'system') delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = theme;
}

class ThemeStore {
  current = $state<Theme>('system');

  init() {
    this.current = read();
    apply(this.current);
  }

  cycle() {
    this.current = order[(order.indexOf(this.current) + 1) % order.length]!;
    apply(this.current);
    try {
      if (this.current === 'system') localStorage.removeItem(KEY);
      else localStorage.setItem(KEY, this.current);
    } catch {
      // Not persisted, still applied for this session.
    }
  }
}

export const theme = new ThemeStore();
