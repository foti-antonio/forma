import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import {
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Worktrees, IndependentTests } from './Reliability';
import { AgentHub, IssueAgent } from './Agents';
import { agents } from './store';
import type {
  AgentPreview,
  AgentRun,
  Issue,
  ProviderStatus,
} from '../../lib/ipc';
vi.mock('./store', async (original) => {
  const actual = await original<typeof import('./store')>();
  return {
    ...actual,
    agents: Object.fromEntries(
      Object.keys(actual.agents).map((key) => [key, vi.fn()]),
    ),
  };
});
const issue = {
  id: 'issue',
  projectId: 'project',
  identifier: 'F1-1',
} as Issue;
const preview: AgentPreview = {
  token: 'single-use',
  prompt: 'Untrusted task: <script>alert(1)</script>',
  provider: 'codex',
  executable: '/trusted/codex',
  version: 'codex-cli test',
  repository: '/repo',
  baseCommit: 'a'.repeat(40),
  policy: 'restricted',
  maxRuntime: 60,
  logLimit: 1024,
  warning: 'Provider sandbox is not Forma OS isolation.',
};
const run: AgentRun = {
  issueIdentifier: 'F1-1',
  issueTitle: 'Task',
  id: 'run',
  issueId: 'issue',
  repositoryId: 'repo',
  provider: 'codex',
  worktree: '/private/worktree',
  branch: 'forma/agent-test',
  baseCommit: 'a'.repeat(40),
  status: 'Running',
  startedAt: new Date().toISOString(),
  endedAt: null,
  exitCode: null,
  result: 'Working',
  log: '{"type":"task.started"}\n<script>inert</script>',
  review: null,
  trusted: true,
};
const provider: ProviderStatus = {
  settings: {
    provider: 'codex',
    executable: '/trusted/codex',
    trusted: true,
    maxRuntime: 60,
    logLimit: 1024,
    policy: 'restricted',
  },
  version: 'test',
  available: true,
  message: 'Installed and authorized',
};
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(agents.concurrency).mockResolvedValue(1);
  vi.mocked(agents.setConcurrency).mockResolvedValue(undefined);
  vi.mocked(agents.testProject).mockResolvedValue('project');
  vi.mocked(agents.testCommand).mockResolvedValue(null);
  vi.mocked(agents.testRuns).mockResolvedValue([]);
  vi.mocked(agents.queue).mockResolvedValue([]);
  vi.mocked(agents.worktrees).mockResolvedValue([]);
  vi.mocked(agents.providers).mockResolvedValue([provider]);
  vi.mocked(agents.preferred).mockResolvedValue('codex');
  vi.mocked(agents.runs).mockResolvedValue([]);
  vi.mocked(agents.recent).mockResolvedValue([]);
  vi.mocked(agents.preview).mockResolvedValue(preview);
  vi.mocked(agents.start).mockResolvedValue(run);
});
afterEach(cleanup);
it('previews saved task as inert text and starts only after explicit approval', async () => {
  const user = userEvent.setup();
  render(<IssueAgent issue={issue} disabled={false} />);
  await user.click(screen.getByText('AI Agent'));
  await screen.findByText('Installed and authorized');
  expect(agents.start).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Preview agent task' }));
  const dialog = await screen.findByRole('dialog');
  expect(
    (
      within(dialog).getByLabelText(
        'Task prompt (saved issue)',
      ) as HTMLTextAreaElement
    ).value,
  ).toContain('<script>');
  expect(document.querySelector('script')).toBeNull();
  expect(agents.start).not.toHaveBeenCalled();
  await user.click(
    within(dialog).getByRole('button', { name: 'Cancel approval' }),
  );
  expect(agents.start).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Preview agent task' }));
  await user.click(
    await screen.findByRole('button', { name: 'Approve & run once' }),
  );
  await waitFor(() =>
    expect(agents.start).toHaveBeenCalledWith('single-use', false),
  );
  expect(agents.preview).toHaveBeenCalledWith('project', 'issue', 'codex');
  await waitFor(() =>
    expect(document.activeElement?.id).toBe('agent-preview-task'),
  );
});
it('requires separate Claude risk consent and preserves approval errors', async () => {
  vi.mocked(agents.preferred).mockResolvedValue('claude');
  vi.mocked(agents.preview).mockResolvedValue({
    ...preview,
    provider: 'claude',
    policy: 'claude-tools',
  });
  vi.mocked(agents.start).mockRejectedValue({
    message: 'Preview expired. Preview again.',
  });
  const user = userEvent.setup();
  render(<IssueAgent issue={issue} disabled={false} />);
  await user.click(screen.getByText('AI Agent'));
  await screen.findByText('Configure executables', { exact: false });
  await waitFor(() =>
    expect(
      (screen.getByLabelText('Coding agent') as HTMLSelectElement).value,
    ).toBe('claude'),
  );
  await user.click(screen.getByRole('button', { name: 'Preview agent task' }));
  const approve = await screen.findByRole('button', {
    name: 'Approve & run once',
  });
  expect((approve as HTMLButtonElement).disabled).toBe(true);
  await user.click(screen.getByRole('checkbox'));
  await user.click(approve);
  expect(
    (await screen.findAllByText('Preview expired. Preview again.')).length,
  ).toBeGreaterThan(0);
  expect(agents.start).toHaveBeenCalledWith('single-use', true);
});
it('shows progress, cancels execution and keeps unsafe output inert', async () => {
  vi.mocked(agents.recent).mockResolvedValue([run]);
  const user = userEvent.setup();
  render(<AgentHub />);
  await user.click(screen.getByRole('button', { name: 'AI agents & runs' }));
  await screen.findByText('codex · Running');
  expect(screen.getByLabelText('Recent progress events').textContent).toContain(
    'task.started',
  );
  expect(document.querySelector('script')).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Cancel execution' }));
  await waitFor(() => expect(agents.cancel).toHaveBeenCalledWith('run'));
});
it('reviews bounded changes, opens worktree and records manual acceptance', async () => {
  vi.mocked(agents.recent).mockResolvedValue([
    { ...run, status: 'Completed', exitCode: 0 },
  ]);
  vi.mocked(agents.review).mockResolvedValue({
    files: ['<img src=x>'],
    diff: '+safe change',
    additions: 1,
    deletions: 0,
    truncated: true,
  });
  const user = userEvent.setup();
  render(<AgentHub />);
  await user.click(screen.getByRole('button', { name: 'AI agents & runs' }));
  await user.click(
    await screen.findByRole('button', { name: 'Inspect result' }),
  );
  await screen.findByText('<img src=x>');
  expect(document.querySelector('img')).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Open worktree' }));
  expect(agents.open).toHaveBeenCalledWith('run');
  await user.click(
    screen.getByRole('button', { name: 'Accept for manual integration' }),
  );
  await waitFor(() =>
    expect(agents.decide).toHaveBeenCalledWith('run', 'accepted'),
  );
});
it('uses executable authorization and persists validated provider defaults', async () => {
  const user = userEvent.setup();
  render(<AgentHub />);
  await user.click(screen.getByRole('button', { name: 'AI agents & runs' }));
  await user.click(screen.getByRole('button', { name: 'Agent settings' }));
  await screen.findByText('/trusted/codex');
  await user.click(
    screen.getByRole('button', {
      name: 'Choose & authorize codex executable…',
    }),
  );
  expect(agents.authorize).toHaveBeenCalledWith('codex');
  await user.clear(screen.getByLabelText('Maximum runtime (seconds)'));
  await user.type(screen.getByLabelText('Maximum runtime (seconds)'), '120');
  await user.click(screen.getByRole('button', { name: 'Save codex defaults' }));
  expect(agents.saveSettings).toHaveBeenCalledWith(
    expect.objectContaining({ maxRuntime: 120, logLimit: 1024 }),
  );
});

it('queues approval without executing and dispatches only through explicit controls', async () => {
  const user = userEvent.setup();
  const view = render(<IssueAgent issue={issue} disabled={false} />);
  await user.click(screen.getByText('AI Agent'));
  await user.click(
    await screen.findByRole('button', { name: 'Preview agent task' }),
  );
  await user.click(
    await screen.findByRole('button', { name: 'Approve & add to queue' }),
  );
  expect(agents.enqueue).toHaveBeenCalledWith('single-use', false);
  expect(agents.start).not.toHaveBeenCalled();
  view.unmount();
  render(<AgentHub />);
  await user.click(screen.getByRole('button', { name: 'AI agents & runs' }));
  await user.click(screen.getByRole('button', { name: 'Approved task queue' }));
  expect(agents.startQueue).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Start Next' }));
  expect(agents.startQueue).toHaveBeenLastCalledWith(false);
  await user.click(
    screen.getByRole('button', { name: 'Start Approved Queue' }),
  );
  expect(agents.startQueue).toHaveBeenLastCalledWith(true);
  await user.click(screen.getByRole('button', { name: 'Stop queue dispatch' }));
  expect(agents.queueStop).toHaveBeenCalledOnce();
});
it('defaults to one and explicitly enables two simultaneous runs', async () => {
  const user = userEvent.setup();
  render(<AgentHub />);
  await user.click(screen.getByRole('button', { name: 'AI agents & runs' }));
  await user.click(screen.getByRole('button', { name: 'Agent settings' }));
  const select = await screen.findByLabelText('Maximum simultaneous runs');
  await waitFor(() =>
    expect((select as HTMLSelectElement).disabled).toBe(false),
  );
  expect((select as HTMLSelectElement).value).toBe('1');
  await user.selectOptions(select, '2');
  expect(agents.setConcurrency).toHaveBeenCalledWith(2);
});
it('requires deletion confirmation for clean reviewed worktrees and retains errors', async () => {
  const user = userEvent.setup();
  vi.mocked(agents.worktrees).mockResolvedValue([
    {
      run: { ...run, status: 'Completed', review: 'accepted' },
      repository: '/repo',
      cleanupState: 'Retained',
      message: '',
      dirty: false,
      diskBytes: 100,
    },
  ]);
  vi.mocked(agents.cleanup).mockRejectedValueOnce(
    new Error('Worktree changed; preserve files'),
  );
  render(<Worktrees />);
  await user.click(
    await screen.findByRole('button', { name: 'Clean up reviewed worktree…' }),
  );
  expect(agents.cleanup).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Keep worktree' }));
  expect(agents.cleanup).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole('button', { name: 'Clean up reviewed worktree…' }),
  );
  await user.click(
    screen.getByRole('button', { name: 'Delete clean worktree' }),
  );
  expect(agents.cleanup).toHaveBeenCalledWith('run');
  expect(
    await within(screen.getByRole('dialog')).findByRole('alert'),
  ).toBeTruthy();
});
it('shows exact test argv and directory and requires a separate execution approval', async () => {
  const user = userEvent.setup();
  const command = {
    projectId: 'project',
    executable: '/trusted/test-runner',
    trusted: true,
    arguments: ['test', '$(touch injected)'],
    maxRuntime: 60,
    logLimit: 1024,
  };
  vi.mocked(agents.testCommand).mockResolvedValue(command);
  vi.mocked(agents.testPreview).mockResolvedValue({
    token: 'test-token',
    runId: 'run',
    command,
    workingDirectory: '/private/worktree',
    expiresAt: 9999999999,
  });
  render(<IndependentTests runId="run" />);
  await user.click(
    await screen.findByRole('button', {
      name: 'Save & preview independent tests',
    }),
  );
  await screen.findByText('Working directory: /private/worktree');
  expect(screen.getByText(/touch injected/, { selector: 'pre' })).toBeTruthy();
  expect(agents.testStart).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole('button', { name: 'Approve & run independent tests' }),
  );
  expect(agents.testStart).toHaveBeenCalledWith('test-token');
});
