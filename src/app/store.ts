import { create } from 'zustand';
import { api, errorMessage } from '../lib/ipc';
import type { Snapshot } from '../lib/ipc';
interface State extends Snapshot {
  loading: boolean;
  busy: boolean;
  error: string | null;
  ready: boolean;
  refresh: () => Promise<void>;
  mutate: (action: () => Promise<Snapshot>) => Promise<boolean>;
  clearError: () => void;
}
export const useAppStore = create<State>((set, get) => ({
  workspaces: [],
  projects: [],
  activeWorkspaceId: null,
  loading: false,
  busy: false,
  ready: false,
  error: null,
  clearError: () => set({ error: null }),
  refresh: async () => {
    if (get().loading || get().busy) return;
    set({ loading: true, error: null });
    try {
      set({ ...(await api.snapshot()), ready: true });
    } catch (error) {
      set({ error: errorMessage(error) });
    } finally {
      set({ loading: false });
    }
  },
  mutate: async (action) => {
    if (get().busy || get().loading) return false;
    set({ busy: true, error: null });
    try {
      set(await action());
      return true;
    } catch (error) {
      set({ error: errorMessage(error) });
      return false;
    } finally {
      set({ busy: false });
    }
  },
}));
