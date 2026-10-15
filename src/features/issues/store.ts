import { create } from 'zustand';
import { statuses } from './options';
import { api, errorMessage } from '../../lib/ipc';
import type {
  Issue,
  IssueInput,
  IssueQuery,
  IssuePage,
  Label,
  LabelInput,
} from '../../lib/ipc';
interface State {
  board: boolean;
  columns: Record<string, IssuePage>;
  offsets: Record<string, number>;
  columnPage: (status: string, offset: number) => Promise<void>;
  query: IssueQuery | null;
  workspaceId: string;
  page: IssuePage;
  labels: Label[];
  loading: boolean;
  busy: boolean;
  error: string | null;
  mutationError: string | null;
  generation: number;
  load: (
    query: IssueQuery,
    workspaceId: string,
    board?: boolean,
  ) => Promise<void>;
  refresh: () => Promise<void>;
  mutate: <T>(action: () => Promise<T>) => Promise<{ value: T } | null>;
  save: (id: string | null, input: IssueInput) => Promise<Issue | null>;
  saveLabel: (id: string | null, input: LabelInput) => Promise<boolean>;
  deleteLabel: (id: string) => Promise<boolean>;
  clearError: () => void;
}
export const emptyPage: IssuePage = {
  issues: [],
  total: 0,
  projectTotal: 0,
  done: 0,
};
export const useIssueStore = create<State>((set, get) => ({
  board: false,
  columns: {},
  offsets: {},
  columnPage: async (status, offset) => {
    set({ offsets: { ...get().offsets, [status]: offset } });
    await get().refresh();
  },
  query: null,
  workspaceId: '',
  page: emptyPage,
  labels: [],
  loading: false,
  busy: false,
  error: null,
  mutationError: null,
  generation: 0,
  clearError: () => set({ mutationError: null }),
  load: async (query, workspaceId, board = false) => {
    if (query !== get().query || board !== get().board) set({ offsets: {} });
    const generation = get().generation + 1;
    const switched = get().query?.projectId !== query.projectId;
    set({
      generation,
      board,
      query,
      workspaceId,
      loading: true,
      error: null,
      ...(switched ? { page: emptyPage, labels: [], columns: {} } : {}),
    });
    try {
      const [pages, labels] = await Promise.all([
        board
          ? Promise.all(
              Object.keys(statuses).map(async (status) => {
                const page =
                  query.status && query.status !== status
                    ? emptyPage
                    : await api.issues({
                        ...query,
                        status: status as IssueQuery['status'],
                        sort: 'position',
                        limit: 25,
                        offset: get().offsets[status] ?? 0,
                      });
                return [status, page] as const;
              }),
            )
          : api.issues(query),
        api.labels(workspaceId),
      ]);
      if (get().generation === generation) {
        if (Array.isArray(pages)) {
          const first =
            pages.find(([, p]) => p !== emptyPage)?.[1] ?? emptyPage;
          set({
            columns: Object.fromEntries(pages),
            labels,
            page: {
              ...first,
              issues: [],
              total: pages.reduce((n, [, p]) => n + p.total, 0),
            },
          });
        } else set({ page: pages, labels });
      }
    } catch (error) {
      if (get().generation === generation) set({ error: errorMessage(error) });
    } finally {
      if (get().generation === generation) set({ loading: false });
    }
  },
  refresh: async () => {
    const { query, workspaceId } = get();
    if (query) await get().load(query, workspaceId, get().board);
  },
  mutate: async (action) => {
    if (get().busy) return null;
    set({ busy: true, mutationError: null });
    try {
      const value = await action();
      await get().refresh();
      return { value };
    } catch (error) {
      set({ mutationError: errorMessage(error) });
      return null;
    } finally {
      set({ busy: false });
    }
  },
  save: async (id, input) =>
    (await get().mutate(() => api.saveIssue(id, input)))?.value ?? null,
  saveLabel: async (id, input) =>
    !!(await get().mutate(() => api.saveLabel(id, input))),
  deleteLabel: async (id) =>
    !!(await get().mutate(() => api.deleteLabel(get().workspaceId, id))),
}));
// Detail reads pass through this store boundary and use scoped, typed IPC.
export const issueDetails = {
  read: api.issue,
  children: api.subtasks,
  activity: api.activity,
  parents: (query: IssueQuery) => api.issues(query),
  archive: (issue: Issue) =>
    useIssueStore
      .getState()
      .mutate(() =>
        api.archiveIssue(issue.projectId, issue.id, !issue.archived),
      ),
  delete: (issue: Issue) =>
    useIssueStore
      .getState()
      .mutate(() => api.deleteIssue(issue.projectId, issue.id)),
};

export const issuePreferences = {
  read: api.projectView,
  save: api.setProjectView,
};
export async function moveBoardIssue(
  issue: Issue,
  status: Issue['status'],
  anchor: string | null,
  after: boolean,
) {
  const store = useIssueStore.getState();
  const result = await store.mutate(() =>
    api.moveIssue(issue.projectId, issue.id, status, anchor, after),
  );
  if (!result) await store.refresh();
  return !!result;
}
