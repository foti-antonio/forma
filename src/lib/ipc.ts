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
  agentProviders: () => command<ProviderStatus[]>('agent_providers'),
  agentPreferred: () => command<string>('agent_preferred'),
  agentSavePreferences: (preferred: string) =>
    command<void>('agent_save_preferences', { preferred }),
  agentSaveSettings: (input: AgentSettings) =>
    command<void>('agent_save_settings', { input }),
  agentAuthorize: (provider: string) =>
    command<boolean>('agent_authorize', { provider }),
  agentPreview: (projectId: string, issueId: string, provider: string) =>
    command<AgentPreview>('agent_preview', { projectId, issueId, provider }),
  testProject: (runId: string) => command<string>('test_project', { runId }),
  agentQueueStop: () => command<void>('agent_queue_stop'),
  agentWorktrees: (offset = 0) =>
    command<WorktreeInfo[]>('agent_worktrees', { offset }),
  agentCleanup: (runId: string) =>
    command<void>('agent_cleanup', { runId, confirmed: true }),
  testCommand: (projectId: string) =>
    command<TestCommand | null>('test_command', { projectId }),
  testAuthorize: (projectId: string) =>
    command<boolean>('test_authorize', { projectId }),
  testSaveCommand: (input: TestCommand) =>
    command<void>('test_save_command', { input }),
  testPreview: (runId: string) =>
    command<TestPreview>('test_preview', { runId }),
  testStart: (token: string) =>
    command<string>('test_start', { token, approved: true }),
  testRuns: (runId: string) =>
    command<IndependentTest[]>('test_runs', { runId }),
  testCancel: (testId: string) => command<void>('test_cancel', { testId }),
  agentConcurrency: () => command<number>('agent_concurrency'),
  agentSetConcurrency: (maximum: number) =>
    command<void>('agent_set_concurrency', { maximum }),
  agentEnqueue: (token: string, elevatedApproved: boolean) =>
    command<string>('agent_enqueue', {
      token,
      approved: true,
      elevatedApproved,
    }),
  agentQueue: () => command<QueuedTask[]>('agent_queue'),
  agentQueueCancel: (taskId: string) =>
    command<void>('agent_queue_cancel', { taskId }),
  agentQueueMove: (taskId: string, before: string | null) =>
    command<void>('agent_queue_move', { taskId, before }),
  agentStartQueue: (all: boolean) =>
    command<AgentRun[]>('agent_start_queue', { all }),
  agentStart: (token: string, elevatedApproved: boolean) =>
    command<AgentRun>('agent_start', {
      token,
      approved: true,
      elevatedApproved,
    }),
  agentRuns: (projectId: string, issueId: string, offset = 0) =>
    command<AgentRun[]>('agent_runs', { projectId, issueId, offset }),
  agentRecent: (offset = 0) => command<AgentRun[]>('agent_recent', { offset }),
  agentCancel: (runId: string) => command<void>('agent_cancel', { runId }),
  agentReview: (runId: string) =>
    command<AgentReview>('agent_review', { runId }),
  agentMarkReviewed: (runId: string, decision: 'accepted' | 'rejected') =>
    command<void>('agent_mark_reviewed', { runId, decision }),
  agentOpenWorktree: (runId: string) =>
    command<void>('agent_open_worktree', { runId }),
  gitConnection: (projectId: string) =>
    command<GitRepository | null>('git_connection', { projectId }),
  gitConnect: (projectId: string) =>
    command<GitRepository | null>('git_connect', { projectId }),
  gitDisconnect: (projectId: string) =>
    command<void>('git_disconnect', { projectId }),
  gitOverview: (projectId: string, repositoryId: string, offset = 0) =>
    command<GitOverview>('git_overview', {
      projectId,
      repositoryId,
      offset,
      limit: 20,
    }),
  gitBranches: (
    projectId: string,
    repositoryId: string,
    search = '',
    offset = 0,
  ) =>
    command<GitBranchPage>('git_branches', {
      projectId,
      repositoryId,
      search,
      offset,
      limit: 25,
    }),
  gitLinks: (projectId: string, issueId: string) =>
    command<GitLink[]>('git_links', { projectId, issueId }),
  gitLink: (input: GitLinkInput) => command<GitLink[]>('git_link', { input }),
  gitUnlink: (projectId: string, issueId: string, linkId: string) =>
    command<GitLink[]>('git_unlink', { projectId, issueId, linkId }),
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

export interface GitRepository {
  id: string;
  projectId: string;
  name: string;
  path: string;
  connected: boolean;
  trusted: boolean;
}
export interface GitCommit {
  hash: string;
  message: string;
  author: string;
  committedAt: string;
}
export interface GitBranch {
  name: string;
  active: boolean;
  commit: GitCommit;
}
export interface GitOverview {
  branch: string;
  bare: boolean;
  status: {
    modified: number;
    added: number;
    deleted: number;
    untracked: number;
    conflicted: number;
  } | null;
  commits: GitCommit[];
  hasMore: boolean;
}
export interface GitBranchPage {
  branches: GitBranch[];
  hasMore: boolean;
  truncated: boolean;
}
export interface GitLink {
  id: string;
  repositoryId: string;
  repositoryName: string;
  repositoryPath: string;
  connected: boolean;
  kind: 'branch' | 'commit';
  target: string;
  commit: GitCommit;
  createdAt: string;
}
export interface GitLinkInput {
  projectId: string;
  issueId: string;
  repositoryId: string;
  kind: 'branch' | 'commit';
  target: string;
  confirmed: boolean;
}

export interface AgentSettings {
  provider: string;
  executable: string | null;
  trusted: boolean;
  maxRuntime: number;
  logLimit: number;
  policy: 'restricted' | 'claude-tools';
  requireOsIsolation?: boolean;
}
export interface ProviderStatus {
  settings: AgentSettings;
  version: string | null;
  available: boolean;
  message: string;
}
export interface AgentPreview {
  token: string;
  prompt: string;
  provider: string;
  executable: string;
  version: string;
  repository: string;
  baseCommit: string;
  policy: string;
  maxRuntime: number;
  logLimit: number;
  warning: string;
}
export interface AgentRun {
  issueIdentifier: string;
  issueTitle: string;
  id: string;
  issueId: string;
  repositoryId: string;
  provider: string;
  worktree: string;
  branch: string;
  baseCommit: string;
  status:
    'Queued' | 'Running' | 'Completed' | 'Failed' | 'Cancelled' | 'Interrupted';
  startedAt: string;
  endedAt: string | null;
  exitCode: number | null;
  result: string;
  log: string;
  review: 'accepted' | 'rejected' | null;
  trusted: boolean;
}
export interface AgentReview {
  files: string[];
  diff: string;
  additions: number;
  deletions: number;
  truncated: boolean;
}

export interface QueuedTask {
  id: string;
  issueId: string;
  title: string;
  provider: string;
  approvedAt: string;
  expiresAt: string;
  position: number;
  status: string;
  message: string;
  runId: string | null;
}

export interface TestCommand {
  projectId: string;
  executable: string;
  trusted: boolean;
  arguments: string[];
  maxRuntime: number;
  logLimit: number;
}
export interface TestPreview {
  token: string;
  runId: string;
  command: TestCommand;
  workingDirectory: string;
  expiresAt: number;
}
export interface IndependentTest {
  id: string;
  runId: string;
  executable: string;
  arguments: string[];
  workingDirectory: string;
  status: string;
  startedAt: string;
  endedAt: string | null;
  durationMs: number | null;
  exitCode: number | null;
  result: string;
  log: string;
}
export interface WorktreeInfo {
  run: AgentRun;
  repository: string;
  cleanupState: string;
  message: string;
  dirty: boolean | null;
  diskBytes: number | null;
}
