import { create } from 'zustand';
import { api, errorMessage } from '../../lib/ipc';
import type {
  GitRepository,
  GitOverview,
  GitBranchPage,
  GitLinkInput,
} from '../../lib/ipc';

interface State {
  projectId: string | null;
  repository: GitRepository | null;
  overview: GitOverview | null;
  branches: GitBranchPage | null;
  loading: boolean;
  busy: boolean;
  error: string | null;
  generation: number;
  load: (
    projectId: string,
    search?: string,
    branchOffset?: number,
    commitOffset?: number,
  ) => Promise<void>;
  connect: (projectId: string) => Promise<void>;
  disconnect: (projectId: string) => Promise<void>;
  reset: () => void;
}
export const useGitStore = create<State>((set, get) => ({
  projectId: null,
  repository: null,
  overview: null,
  branches: null,
  loading: false,
  busy: false,
  error: null,
  generation: 0,
  reset: () =>
    set({
      projectId: null,
      repository: null,
      overview: null,
      branches: null,
      loading: false,
      error: null,
      generation: get().generation + 1,
    }),
  load: async (projectId, search = '', branchOffset = 0, commitOffset = 0) => {
    const generation = get().generation + 1;
    set({
      projectId,
      generation,
      loading: true,
      error: null,
      overview: get().projectId === projectId ? get().overview : null,
      branches: get().projectId === projectId ? get().branches : null,
      repository: get().projectId === projectId ? get().repository : null,
    });
    try {
      const repository = await api.gitConnection(projectId);
      if (get().generation !== generation) return;
      set({
        repository,
        ...(!repository?.trusted ? { overview: null, branches: null } : {}),
      });
      if (repository?.trusted) {
        const [overview, branches] = await Promise.all([
          api.gitOverview(projectId, repository.id, commitOffset),
          api.gitBranches(projectId, repository.id, search, branchOffset),
        ]);
        if (get().generation === generation) set({ overview, branches });
      }
    } catch (e) {
      if (get().generation === generation)
        set({ error: errorMessage(e), overview: null, branches: null });
    } finally {
      if (get().generation === generation) set({ loading: false });
    }
  },
  connect: async (projectId) => {
    if (get().busy) return;
    set({ busy: true, error: null });
    try {
      const repository = await api.gitConnect(projectId);
      if (repository && get().projectId === projectId)
        await get().load(projectId);
    } catch (e) {
      if (get().projectId === projectId) set({ error: errorMessage(e) });
    } finally {
      set({ busy: false });
    }
  },
  disconnect: async (projectId) => {
    if (get().busy) return;
    set({ busy: true, error: null });
    try {
      await api.gitDisconnect(projectId);
      if (get().projectId === projectId) await get().load(projectId);
    } catch (e) {
      if (get().projectId === projectId) set({ error: errorMessage(e) });
    } finally {
      set({ busy: false });
    }
  },
}));
// Detail reads/mutations stay behind the feature store boundary. Each mounted
// issue activity view owns its result and discards obsolete reads on unmount.
export const gitActivity = {
  links: api.gitLinks,
  connection: api.gitConnection,
  branches: api.gitBranches,
  overview: api.gitOverview,
  save: (input: GitLinkInput) => api.gitLink(input),
  remove: api.gitUnlink,
};
// Identifiers must be complete tokens; F1-1 must never match F1-10 or xF1-1.
export function mentionsIssue(text: string, identifier: string): boolean {
  return new RegExp('(^|[^A-Za-z0-9])' + identifier + '(?![A-Za-z0-9])').test(
    text,
  );
}
