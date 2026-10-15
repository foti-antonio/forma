import { create } from 'zustand';
import { api, errorMessage } from '../../lib/ipc';
import { useAppStore } from '../../app/store';
import { emptyPage, useIssueStore } from '../issues/store';

interface State {
  busy: boolean;
  prepared: boolean;
  error: string | null;
  message: string;
  export: () => Promise<void>;
  prepare: () => Promise<void>;
  cancel: () => Promise<void>;
  restore: () => Promise<boolean>;
}
export const useBackupStore = create<State>((set, get) => ({
  busy: false,
  prepared: false,
  error: null,
  message: '',
  export: async () => {
    if (get().busy) return;
    set({ busy: true, error: null, message: '' });
    try {
      if (await api.exportBackup())
        set({
          message:
            'Backup exported. Store it securely; backups are not encrypted.',
        });
    } catch (e) {
      set({ error: errorMessage(e) });
    } finally {
      set({ busy: false });
    }
  },
  prepare: async () => {
    if (get().busy) return;
    set({ busy: true, error: null, message: '' });
    try {
      set({ prepared: await api.prepareRestore() });
    } catch (e) {
      set({ error: errorMessage(e) });
    } finally {
      set({ busy: false });
    }
  },
  cancel: async () => {
    if (get().busy) return;
    set({ busy: true });
    try {
      await api.cancelRestore();
      set({ prepared: false });
    } catch (e) {
      set({ error: errorMessage(e) });
    } finally {
      set({ busy: false });
    }
  },
  restore: async () => {
    if (get().busy || !get().prepared) return false;
    set({ busy: true, error: null });
    try {
      const recovery = await api.restoreBackup();
      const issues = useIssueStore.getState();
      useIssueStore.setState({
        query: null,
        generation: issues.generation + 1,
        page: emptyPage,
        columns: {},
        offsets: {},
        labels: [],
        error: null,
        mutationError: null,
      });
      set({
        prepared: false,
        message: 'Data restored. Recovery copy: ' + recovery,
      });
      await useAppStore.getState().refresh();
      return true;
    } catch (e) {
      set({ prepared: false, error: errorMessage(e) });
      return false;
    } finally {
      set({ busy: false });
    }
  },
}));
