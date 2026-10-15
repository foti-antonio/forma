import { beforeEach, describe, expect, it, vi } from 'vitest';
import { api } from '../lib/ipc';
import type { Snapshot } from '../lib/ipc';
import { useAppStore } from './store';
vi.mock('../lib/ipc', async (original) => ({
  ...(await original<typeof import('../lib/ipc')>()),
  api: { snapshot: vi.fn() },
}));
const snapshot: Snapshot = {
  workspaces: [{ id: 'a', name: 'Studio', createdAt: '', updatedAt: '' }],
  activeWorkspaceId: 'a',
  projects: [],
};
beforeEach(() => {
  useAppStore.setState({
    workspaces: [],
    activeWorkspaceId: null,
    projects: [],
    busy: false,
    loading: false,
    error: null,
    ready: false,
  });
});
describe('application state', () => {
  it('restores the backend snapshot, including the persisted selection', async () => {
    vi.mocked(api.snapshot).mockResolvedValue(snapshot);
    await useAppStore.getState().refresh();
    expect(useAppStore.getState()).toMatchObject({
      ...snapshot,
      ready: true,
      loading: false,
    });
  });
  it('keeps saved state and reports a structured mutation failure', async () => {
    useAppStore.setState(snapshot);
    expect(
      await useAppStore
        .getState()
        .mutate(() =>
          Promise.reject({ code: 'database', message: 'Disk full' }),
        ),
    ).toBe(false);
    expect(useAppStore.getState()).toMatchObject({
      ...snapshot,
      busy: false,
      error: 'Disk full',
    });
  });
  it('serializes mutations and applies the server response', async () => {
    let complete!: (data: Snapshot) => void;
    const first = useAppStore.getState().mutate(
      () =>
        new Promise((resolve) => {
          complete = resolve;
        }),
    );
    const second = vi.fn();
    expect(await useAppStore.getState().mutate(second)).toBe(false);
    expect(second).not.toHaveBeenCalled();
    complete(snapshot);
    expect(await first).toBe(true);
    expect(useAppStore.getState()).toMatchObject({ ...snapshot, busy: false });
  });
  it('allows retry after an initial read failure', async () => {
    vi.mocked(api.snapshot)
      .mockRejectedValueOnce(new Error('Unavailable'))
      .mockResolvedValueOnce(snapshot);
    await useAppStore.getState().refresh();
    expect(useAppStore.getState()).toMatchObject({
      ready: false,
      loading: false,
      error: 'Unavailable',
    });
    await useAppStore.getState().refresh();
    expect(useAppStore.getState()).toMatchObject({ ready: true, error: null });
  });
});
