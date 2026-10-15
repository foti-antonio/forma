import { expect, it } from 'vitest';
import { applyTheme, readTheme } from './theme';
it('honors system appearance and persists explicit choices', () => {
  applyTheme('system', true);
  expect(document.documentElement.dataset.theme).toBe('dark');
  applyTheme('system', false);
  expect(document.documentElement.dataset.theme).toBe('light');
  applyTheme('dark', false);
  expect(readTheme()).toBe('dark');
  expect(document.documentElement.dataset.theme).toBe('dark');
  applyTheme('light', true);
  expect(document.documentElement.dataset.theme).toBe('light');
});
