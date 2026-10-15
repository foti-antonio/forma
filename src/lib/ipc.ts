import { invoke, isTauri } from '@tauri-apps/api/core';
export interface Workspace {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
}
export type Status = 'planned' | 'active' | 'paused' | 'completed';
export type ProjectIcon =
  'layers' | 'code' | 'rocket' | 'box' | 'globe' | 'sparkles';
export interface ProjectInput {
  workspaceId: string;
  name: string;
  description: string;
  icon: ProjectIcon;
  color: string;
  status: Status;
}
export interface Project extends ProjectInput {
  id: string;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface Snapshot {
  theme?: 'light' | 'dark' | 'system' | null;
  workspaces: Workspace[];
  activeWorkspaceId: string | null;
  projects: Project[];
}
export interface AppError {
  code: string;
  message: string;
}
export function errorMessage(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  )
    return error.message;
  return typeof error === 'string'
    ? error
    : 'Something went wrong. Please try again.';
}
async function command<T>(
  name: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauri())
    throw {
      code: 'desktop_required',
      message:
        'Open Forma as a desktop app to access your local database. Run pnpm tauri dev from the project directory.',
    };
  return invoke<T>(name, args);
}
export const api = {
  setTheme: (theme: 'light' | 'dark' | 'system') =>
    command<Snapshot>('set_theme', { theme }),
  projectView: (projectId: string) =>
    command<'list' | 'board'>('project_view', { projectId }),
  setProjectView: (projectId: string, view: 'list' | 'board') =>
    command<void>('set_project_view', { projectId, view }),
  moveIssue: (
    projectId: string,
    id: string,
    status: IssueStatus,
    before: string | null,
    after = false,
  ) => command<void>('move_issue', { projectId, id, status, before, after }),
  exportBackup: () => command<boolean>('export_backup'),
  prepareRestore: () => command<boolean>('prepare_restore'),
  restoreBackup: () => command<string>('restore_backup', { confirmed: true }),
  cancelRestore: () => command<void>('cancel_restore'),
  issues: (query: IssueQuery) => command<IssuePage>('issues', { query }),
  issue: (projectId: string, id: string) =>
    command<Issue>('issue', { projectId, id }),
  saveIssue: (id: string | null, input: IssueInput) =>
    command<Issue>('save_issue', { id, input }),
  archiveIssue: (projectId: string, id: string, archived: boolean) =>
    command<void>('archive_issue', { projectId, id, archived }),
  deleteIssue: (projectId: string, id: string) =>
    command<void>('delete_issue', { projectId, id }),
  labels: (workspaceId: string) => command<Label[]>('labels', { workspaceId }),
  saveLabel: (id: string | null, input: LabelInput) =>
    command<Label[]>('save_label', { id, input }),
  deleteLabel: (workspaceId: string, id: string) =>
    command<Label[]>('delete_label', { workspaceId, id }),
  activity: (projectId: string, id: string, offset = 0) =>
    command<ActivityPage>('issue_activity', {
      projectId,
      id,
      offset,
      limit: 20,
    }),
  subtasks: (projectId: string, id: string, offset = 0) =>
    command<IssuePage>('subtasks', { projectId, id, offset, limit: 20 }),
  snapshot: () => command<Snapshot>('snapshot'),
  createWorkspace: (name: string) =>
    command<Snapshot>('create_workspace', { name }),
  renameWorkspace: (id: string, name: string) =>
    command<Snapshot>('rename_workspace', { id, name }),
  selectWorkspace: (id: string) =>
    command<Snapshot>('select_workspace', { id }),
  deleteWorkspace: (id: string) =>
    command<Snapshot>('delete_workspace', { id }),
  saveProject: (id: string | null, input: ProjectInput) =>
    command<Snapshot>('save_project', { id, input }),
  archiveProject: (id: string, archived: boolean) =>
    command<Snapshot>('archive_project', { id, archived }),
  deleteProject: (id: string) => command<Snapshot>('delete_project', { id }),
};

export type IssueStatus =
  'backlog' | 'todo' | 'in_progress' | 'done' | 'cancelled';
export type Priority = 'urgent' | 'high' | 'medium' | 'low' | 'none';
export interface IssueInput {
  projectId: string;
  title: string;
  description: string;
  status: IssueStatus;
  priority: Priority;
  dueDate: string | null;
  parentId: string | null;
  labelIds: string[];
  position: number;
}
export interface Issue extends IssueInput {
  subtaskTotal?: number;
  subtaskDone?: number;
  id: string;
  identifier: string;
  number: number;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface LabelInput {
  workspaceId: string;
  name: string;
  color: string;
}
export interface Label extends LabelInput {
  id: string;
  createdAt: string;
  updatedAt: string;
}
export interface IssueQuery {
  labelId?: string | null;
  dueFrom?: string | null;
  dueTo?: string | null;
  undated?: boolean;
  projectId: string;
  search: string;
  status: IssueStatus | null;
  priority: Priority | null;
  archived: boolean;
  sort: 'position' | 'newest' | 'updated' | 'due' | 'priority';
  offset: number;
  limit: number;
}
export interface IssuePage {
  issues: Issue[];
  total: number;
  projectTotal: number;
  done: number;
}
export interface Activity {
  id: string;
  kind: string;
  summary: string;
  createdAt: string;
}
export interface ActivityPage {
  events: Activity[];
  total: number;
}
