import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { api } from '../../lib/ipc';
import type { Issue, GitRepository, GitCommit } from '../../lib/ipc';
import { GitPanel } from './GitPanel';
import { IssueGitActivity } from './IssueGitActivity';
import { mentionsIssue, useGitStore } from './store';
vi.mock('../../lib/ipc', async (original) => ({
  ...(await original<typeof import('../../lib/ipc')>()),
  api: {
    gitConnection: vi.fn(),
    gitConnect: vi.fn(),
    gitDisconnect: vi.fn(),
    gitOverview: vi.fn(),
    gitBranches: vi.fn(),
    gitLinks: vi.fn(),
    gitLink: vi.fn(),
    gitUnlink: vi.fn(),
  },
}));
const repo: GitRepository = {
  id: 'repo',
  projectId: 'project',
  name: 'Engine',
  path: '/local/engine',
  connected: true,
  trusted: true,
};
const commit: GitCommit = {
  hash: 'a'.repeat(40),
  message: 'Fix F1-1\n<script>inert</script>',
  author: 'Local Author',
  committedAt: '2026-10-15T10:00:00Z',
};
const issue = {
  id: 'issue',
  projectId: 'project',
  identifier: 'F1-1',
} as Issue;
beforeEach(() => {
  vi.resetAllMocks();
  useGitStore.getState().reset();
  useGitStore.setState({ busy: false });
  vi.mocked(api.gitConnection).mockResolvedValue(repo);
  vi.mocked(api.gitConnect).mockResolvedValue(repo);
  vi.mocked(api.gitOverview).mockResolvedValue({
    branch: 'main',
    bare: false,
    status: { modified: 2, added: 1, deleted: 0, untracked: 3, conflicted: 0 },
    commits: [commit],
    hasMore: false,
  });
  vi.mocked(api.gitBranches).mockResolvedValue({
    branches: [{ name: 'feature/F1-1-work', active: false, commit }],
    hasMore: false,
    truncated: false,
  });
  vi.mocked(api.gitLinks).mockResolvedValue([]);
  vi.mocked(api.gitLink).mockResolvedValue([
    {
      id: 'link',
      repositoryId: repo.id,
      repositoryName: repo.name,
      repositoryPath: repo.path,
      connected: true,
      kind: 'branch',
      target: 'feature/F1-1-work',
      commit,
      createdAt: commit.committedAt,
    },
  ]);
});
afterEach(cleanup);
it('connects through native selection, renders bounded status/history, refreshes and confirms disconnect', async () => {
  vi.mocked(api.gitConnection).mockResolvedValueOnce(null);
  const user = userEvent.setup();
  render(<GitPanel projectId="project" />);
  await screen.findByText(/Connect an existing repository/);
  await user.click(screen.getByRole('button', { name: 'Connect repository…' }));
  await screen.findByText('/local/engine');
  expect(screen.getByText(/2 modified/).textContent).toContain('3 untracked');
  expect(screen.getByText('main')).toBeTruthy();
  expect(screen.getAllByText(/Fix F1-1.*<script>inert<\/script>/s).length).toBe(
    2,
  );
  expect(document.querySelector('script')).toBeNull();
  expect(api.gitOverview).toHaveBeenCalledWith('project', 'repo', 0);
  await user.click(screen.getByRole('button', { name: 'Refresh Git' }));
  await waitFor(() => expect(api.gitOverview).toHaveBeenCalledTimes(2));
  await user.click(screen.getByRole('button', { name: 'Disconnect' }));
  await screen.findByRole('dialog', { name: 'Disconnect repository?' });
  expect(api.gitDisconnect).not.toHaveBeenCalled();
  vi.mocked(api.gitConnection).mockResolvedValue(null);
  await user.click(
    screen.getByRole('button', { name: 'Disconnect repository' }),
  );
  await screen.findByText(/Connect an existing repository/);
  expect(api.gitDisconnect).toHaveBeenCalledWith('project');
});
it('shows cancellation, Git failure and retry, and blocks automatic access to restored paths', async () => {
  vi.mocked(api.gitConnection).mockResolvedValueOnce(null);
  vi.mocked(api.gitConnect).mockResolvedValueOnce(null);
  const user = userEvent.setup();
  render(<GitPanel projectId="project" />);
  await screen.findByText(/Connect an existing/);
  await user.click(screen.getByRole('button', { name: 'Connect repository…' }));
  expect(api.gitOverview).not.toHaveBeenCalled();
  vi.mocked(api.gitOverview).mockRejectedValueOnce(
    new Error('Repository unavailable'),
  );
  await user.click(screen.getByRole('button', { name: 'Refresh Git' }));
  expect((await screen.findByRole('alert')).textContent).toContain(
    'Repository unavailable',
  );
  await user.click(screen.getByRole('button', { name: 'Refresh Git' }));
  await screen.findByText('main');
  vi.mocked(api.gitConnection).mockResolvedValue({ ...repo, trusted: false });
  const count = vi.mocked(api.gitOverview).mock.calls.length;
  await user.click(screen.getByRole('button', { name: 'Refresh Git' }));
  await screen.findByText(/Restored connection/);
  expect(api.gitOverview).toHaveBeenCalledTimes(count);
});
it('filters and pages branches without losing search focus and uses bounded history pages', async () => {
  vi.mocked(api.gitBranches).mockResolvedValue({
    branches: [{ name: 'main', active: true, commit }],
    hasMore: true,
    truncated: false,
  });
  vi.mocked(api.gitOverview).mockResolvedValue({
    branch: 'main',
    bare: true,
    status: null,
    commits: [commit],
    hasMore: true,
  });
  const user = userEvent.setup();
  render(<GitPanel projectId="project" />);
  await screen.findByText(/Bare repository/);
  const search = screen.getByRole('searchbox', { name: 'Find branches' });
  await user.type(search, 'feat');
  await waitFor(() =>
    expect(api.gitBranches).toHaveBeenLastCalledWith(
      'project',
      'repo',
      'feat',
      0,
    ),
  );
  expect(document.activeElement).toBe(search);
  await user.click(screen.getByRole('button', { name: 'Next branches' }));
  await waitFor(() =>
    expect(api.gitBranches).toHaveBeenLastCalledWith(
      'project',
      'repo',
      'feat',
      25,
    ),
  );
  await user.click(screen.getByRole('button', { name: 'Older commits' }));
  await waitFor(() =>
    expect(api.gitOverview).toHaveBeenLastCalledWith('project', 'repo', 20),
  );
});
it('suggests exact identifiers, requires confirmation, keeps failures reviewable and removes links explicitly', async () => {
  const user = userEvent.setup();
  render(<IssueGitActivity issue={issue} disabled={false} />);
  await user.click(screen.getByText('Git activity', { selector: 'summary' }));
  await screen.findByText(/No Git work linked/);
  expect(api.gitLink).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole('button', { name: 'Suggested branch: feature/F1-1-work' }),
  );
  await screen.findByRole('alertdialog', { name: 'Confirm Git association' });
  expect(api.gitLink).not.toHaveBeenCalled();
  vi.mocked(api.gitLink).mockRejectedValueOnce(new Error('Branch was deleted'));
  await user.click(screen.getByRole('button', { name: 'Confirm Git link' }));
  expect((await screen.findByRole('alert')).textContent).toContain(
    'Branch was deleted',
  );
  await user.click(screen.getByRole('button', { name: 'Confirm Git link' }));
  await screen.findByText('branch: feature/F1-1-work');
  expect(api.gitLink).toHaveBeenLastCalledWith({
    projectId: 'project',
    issueId: 'issue',
    repositoryId: 'repo',
    kind: 'branch',
    target: 'feature/F1-1-work',
    confirmed: true,
  });
  await user.click(screen.getByRole('button', { name: 'Remove Git link' }));
  expect(api.gitUnlink).not.toHaveBeenCalled();
  vi.mocked(api.gitUnlink).mockResolvedValue([]);
  await user.click(screen.getByRole('button', { name: 'Confirm removal' }));
  await screen.findByText(/No Git work linked/);
});
it('retains saved historical links when repository inspection fails', async () => {
  vi.mocked(api.gitLinks).mockResolvedValue([
    {
      id: 'old',
      repositoryId: 'repo',
      repositoryName: 'Engine',
      repositoryPath: '/local/engine',
      connected: false,
      kind: 'branch',
      target: 'deleted-branch',
      commit,
      createdAt: commit.committedAt,
    },
  ]);
  vi.mocked(api.gitOverview).mockRejectedValue(new Error('Missing repository'));
  const user = userEvent.setup();
  render(<IssueGitActivity issue={issue} disabled={false} />);
  await user.click(screen.getByText('Git activity', { selector: 'summary' }));
  await screen.findByRole('alert');
  expect(screen.getByText('branch: deleted-branch')).toBeTruthy();
  expect(screen.getByText(/Disconnected/)).toBeTruthy();
});
it('rejects identifier prefixes, embedded tokens and different project keys', () => {
  expect(mentionsIssue('feature/F1-1-work', 'F1-1')).toBe(true);
  expect(mentionsIssue('Fix (F1-1).', 'F1-1')).toBe(true);
  for (const text of ['F1-10', 'F11-1', 'xF1-1', 'F1-1x', 'f1-1'])
    expect(mentionsIssue(text, 'F1-1')).toBe(false);
});
it('discards a response from a previously selected project', async () => {
  let resolve!: (value: GitRepository) => void;
  vi.mocked(api.gitConnection).mockImplementationOnce(
    () =>
      new Promise((r) => {
        resolve = r;
      }),
  );
  const old = useGitStore.getState().load('old');
  vi.mocked(api.gitConnection).mockResolvedValueOnce(null);
  await useGitStore.getState().load('new');
  resolve({ ...repo, projectId: 'old' });
  await old;
  expect(useGitStore.getState().projectId).toBe('new');
  expect(useGitStore.getState().repository).toBeNull();
  expect(api.gitOverview).not.toHaveBeenCalled();
});
