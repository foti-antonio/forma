export type Theme = 'system' | 'light' | 'dark';
export function readTheme(): Theme {
  try {
    const value = window.localStorage.getItem('forma.theme');
    return value === 'light' || value === 'dark' ? value : 'system';
  } catch {
    return 'system';
  }
}
export function applyTheme(theme: Theme, prefersDark: boolean) {
  document.documentElement.dataset.theme =
    theme === 'system' ? (prefersDark ? 'dark' : 'light') : theme;
  try {
    window.localStorage.setItem('forma.theme', theme);
  } catch {
    /* Theme remains usable when local storage is disabled. */
  }
}
