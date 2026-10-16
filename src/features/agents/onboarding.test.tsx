import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import {
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Onboarding } from './Onboarding';
import { IssueAgent } from './Agents';
import { agents } from './store';
import { nextRequirement, agentExplanation } from './onboarding-state';
import type { OnboardingStatus, Issue, AgentPreview } from '../../lib/ipc';
vi.mock('./store', async (original) => {
  const actual = await original<typeof import('./store')>();
  return {
    ...actual,
    agents: Object.fromEntries(
      Object.keys(actual.agents).map((key) => [key, vi.fn()]),
    ),
  };
});
const found: OnboardingStatus = {
  provider: 'codex',
  platform: 'linux',
  candidates: [
    { token: 'candidate-id', path: '/trusted/codex', sha256: 'a'.repeat(64) },
  ],
  authorized: false,
  compatible: false,
  authentication: 'unavailable',
  ready: false,
  diagnostics: 'Select and authorize the provider executable.',
  prerequisites: ['node', 'npm'],
  login: 'idle',
};
const ready: OnboardingStatus = {
  ...found,
  authorized: true,
  compatible: true,
  authentication: 'authenticated',
  ready: true,
  diagnostics: '',
};
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(agents.onboarding).mockResolvedValue(found);
  vi.mocked(agents.connection).mockResolvedValue(null);
  vi.mocked(agents.runs).mockResolvedValue([]);
  vi.mocked(agents.preferred).mockResolvedValue('codex');
  vi.mocked(agents.providers).mockResolvedValue([
    {
      settings: {
        provider: 'codex',
        executable: '/trusted/codex',
        trusted: true,
        maxRuntime: 600,
        logLimit: 65536,
        policy: 'restricted',
      },
      version: 'fixture',
      available: true,
      message: 'Authorized',
    },
  ]);
});
afterEach(cleanup);
it('models missing, incompatible, unauthenticated, unknown, disconnected and blocked states', () => {
  expect(nextRequirement({ ...found, candidates: [] }, false)).toBe(
    'installation',
  );
  expect(nextRequirement({ ...found, authorized: true }, true)).toBe(
    'installation',
  );
  for (const authentication of [
    'unauthenticated',
    'unknown',
    'unavailable',
  ] as const) {
    expect(
      nextRequirement({ ...ready, authentication, ready: false }, true),
    ).toBe('authentication');
  }
  expect(nextRequirement(ready, false)).toBe('repository');
  expect(nextRequirement({ ...ready, ready: false }, true)).toBe('permissions');
  expect(nextRequirement(ready, true)).toBe('ready');
});
it('discovery never authorizes or logs in until the user acts, and rescan exposes new state', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  render(<Onboarding onClose={close} />);
  await user.click(screen.getByRole('button', { name: 'Find installation' }));
  expect(agents.authorizeDiscovered).not.toHaveBeenCalled();
  expect(agents.login).not.toHaveBeenCalled();
  expect(screen.getByText('SHA-256:', { exact: false }).textContent).toContain(
    'a'.repeat(64),
  );
  vi.mocked(agents.onboarding).mockResolvedValue(ready);
  await user.click(
    screen.getByRole('button', {
      name: 'Authorize this installation: /trusted/codex',
    }),
  );
  expect(agents.authorizeDiscovered).toHaveBeenCalledWith('candidate-id');
  expect(
    screen.getByRole('heading', { name: 'Conservative permissions' }),
  ).toBeTruthy();
  expect(agents.start).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Close setup' }));
  expect(close).toHaveBeenCalledOnce();
});
it('missing installations offer official guidance, explicit rescan and a native fallback', async () => {
  vi.mocked(agents.onboarding).mockResolvedValue({ ...found, candidates: [] });
  const user = userEvent.setup();
  render(<Onboarding onClose={vi.fn()} />);
  await user.click(screen.getByRole('button', { name: 'Find installation' }));
  expect(screen.getByText(/No supported executable found/)).toBeTruthy();
  expect(
    (
      screen.getByRole('button', {
        name: 'Continue to sign-in',
      }) as HTMLButtonElement
    ).disabled,
  ).toBe(true);
  await user.click(
    screen.getByRole('button', { name: 'Open official installation guide' }),
  );
  expect(agents.setupDocs).toHaveBeenCalledWith('codex');
  vi.mocked(agents.onboarding).mockResolvedValue(found);
  await user.click(
    screen.getByRole('button', { name: 'Rescan installations' }),
  );
  expect(
    await screen.findByRole('button', {
      name: 'Authorize this installation: /trusted/codex',
    }),
  ).toBeTruthy();
  expect(agents.authorizeDiscovered).not.toHaveBeenCalled();
});
it('unauthenticated and unknown sessions block readiness; login and cancellation are explicit', async () => {
  vi.mocked(agents.onboarding).mockResolvedValue({
    ...ready,
    authentication: 'unauthenticated',
    ready: false,
  });
  const user = userEvent.setup();
  render(<Onboarding onClose={vi.fn()} />);
  await user.click(screen.getByRole('button', { name: 'Find installation' }));
  expect(
    (
      screen.getByRole('button', {
        name: 'Continue',
      }) as HTMLButtonElement
    ).disabled,
  ).toBe(true);
  await user.click(screen.getByRole('button', { name: 'Open Codex login' }));
  expect(agents.login).toHaveBeenCalledWith('codex');
  await user.click(screen.getByRole('button', { name: 'Cancel login' }));
  expect(agents.cancelLogin).toHaveBeenCalledOnce();
  expect(agents.start).not.toHaveBeenCalled();
});
it('guides the first task into the existing exact prompt and explicit execution approval', async () => {
  vi.mocked(agents.onboarding).mockResolvedValue(ready);
  const repo = {
    id: 'repo',
    projectId: 'project',
    path: '/repo',
    name: 'repo',
    trusted: true,
    connected: true,
  };
  vi.mocked(agents.connect).mockResolvedValue(repo);
  const preview = {
    token: 'existing-preview',
    provider: 'codex',
    prompt: 'Exact untrusted prompt <script>inert</script>',
    executable: '/trusted/codex',
    version: 'fixture',
    repository: '/repo',
    baseCommit: 'abc',
    policy: 'restricted',
    maxRuntime: 600,
    logLimit: 65536,
    warning: 'Worktrees are not OS sandboxes.',
  } satisfies AgentPreview;
  vi.mocked(agents.preview).mockResolvedValue(preview);
  const user = userEvent.setup();
  render(
    <IssueAgent
      issue={
        {
          id: 'issue',
          projectId: 'project',
          identifier: 'F1-1',
          archived: false,
        } as Issue
      }
      disabled={false}
    />,
  );
  await user.click(screen.getByRole('button', { name: 'Run with AI' }));
  await user.click(screen.getByRole('button', { name: 'Find installation' }));
  expect(
    (
      screen.getByRole('button', {
        name: 'Continue to permissions',
      }) as HTMLButtonElement
    ).disabled,
  ).toBe(true);
  vi.mocked(agents.connection).mockResolvedValue(repo);
  await user.click(screen.getByRole('button', { name: 'Select Git folder…' }));
  expect(agents.connect).toHaveBeenCalledWith('project');
  expect(agents.saveSettings).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole('button', { name: 'Apply recommended configuration' }),
  );
  expect(agents.saveSettings).toHaveBeenCalledWith(
    expect.objectContaining({
      policy: 'restricted',
      maxRuntime: 600,
      logLimit: 65536,
    }),
  );
  expect(agents.setConcurrency).toHaveBeenCalledWith(1);
  expect(agents.preview).not.toHaveBeenCalled();
  expect(agents.start).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Preview first task' }));
  const dialog = await screen.findByRole('dialog', {
    name: 'Approve codex execution?',
  });
  expect(
    (
      within(dialog).getByLabelText(
        'Task prompt (saved issue)',
      ) as HTMLTextAreaElement
    ).value,
  ).toBe(preview.prompt);
  expect(document.querySelector('script')).toBeNull();
  expect(agents.start).not.toHaveBeenCalled();
  await user.click(
    within(dialog).getByRole('button', { name: 'Approve & run once' }),
  );
  await waitFor(() =>
    expect(agents.start).toHaveBeenCalledWith('existing-preview', false),
  );
});
it('has keyboard provider selection, back navigation, and no execution on Escape', async () => {
  const close = vi.fn();
  const user = userEvent.setup();
  render(<Onboarding onClose={close} />);
  screen.getByLabelText('Choose your AI provider').focus();
  await user.selectOptions(
    screen.getByLabelText('Choose your AI provider'),
    'claude',
  );
  await user.tab();
  await user.keyboard('{Enter}');
  expect(agents.onboarding).toHaveBeenCalledWith('claude');
  await user.click(await screen.findByRole('button', { name: 'Back' }));
  await user.keyboard('{Escape}');
  expect(close).toHaveBeenCalled();
  expect(agents.start).not.toHaveBeenCalled();
});
it.each([
  'Not installed or not on PATH',
  'Installed CLI lacks required security options',
  'Authentication missing',
  'Executable changed since authorization',
  'Connect a repository first',
  'Repository has local changes',
  'Whole-provider OS isolation is required',
  'Maximum runtime exceeded',
  'Agent exited unsuccessfully',
])('makes errors actionable: %s', (details) => {
  expect(agentExplanation(details)).not.toBe(details);
});
it('recommended configuration preserves a required OS isolation block', async () => {
  vi.mocked(agents.onboarding).mockResolvedValue({
    ...ready,
    ready: false,
    diagnostics: 'Whole-provider OS isolation is required.',
  });
  const settings = (await agents.providers())[0].settings;
  vi.mocked(agents.providers).mockResolvedValue([
    {
      settings: { ...settings, requireOsIsolation: true },
      version: 'fixture',
      available: true,
      message: 'Blocked',
    },
  ]);
  const user = userEvent.setup();
  render(<Onboarding onClose={vi.fn()} />);
  await user.click(screen.getByRole('button', { name: 'Find installation' }));
  await user.click(
    screen.getByRole('button', { name: 'Apply recommended configuration' }),
  );
  expect(agents.saveSettings).toHaveBeenCalledWith(
    expect.objectContaining({ requireOsIsolation: true, policy: 'restricted' }),
  );
  await user.click(screen.getByRole('button', { name: 'Finish setup' }));
  expect(
    screen.getByText('Complete the highlighted requirement before continuing.'),
  ).toBeTruthy();
  expect(agents.preview).not.toHaveBeenCalled();
  expect(agents.start).not.toHaveBeenCalled();
});

it('skips completed requirements and reports readiness without reauthorizing or starting tasks', async () => {
  vi.mocked(agents.onboarding).mockResolvedValue(ready);
  vi.mocked(agents.connection).mockResolvedValue({
    id: 'repo',
    projectId: 'project',
    path: '/repo',
    name: 'repo',
    trusted: true,
    connected: true,
  });
  const user = userEvent.setup();
  render(<Onboarding projectId="project" onClose={vi.fn()} />);
  await user.click(screen.getByRole('button', { name: 'Find installation' }));
  expect(
    screen.getByRole('heading', { name: 'Conservative permissions' }),
  ).toBeTruthy();
  const summary = screen.getByRole('list', { name: 'AI readiness' });
  for (const label of [
    'Installed: Yes',
    'Signed in: Yes',
    'Repository connected: Yes',
    'Ready to run: Ready for your approval',
  ]) {
    expect(within(summary).getByText(label)).toBeTruthy();
  }
  expect(agents.authorizeDiscovered).not.toHaveBeenCalled();
  expect(agents.login).not.toHaveBeenCalled();
  expect(agents.connect).not.toHaveBeenCalled();
  expect(agents.start).not.toHaveBeenCalled();
});

it('checks sign-in automatically after the explicitly opened browser flow finishes', async () => {
  vi.mocked(agents.onboarding).mockResolvedValue({
    ...ready,
    authentication: 'unauthenticated',
    ready: false,
  });
  vi.mocked(agents.loginStatus).mockResolvedValue('finished');
  const user = userEvent.setup();
  render(<Onboarding onClose={vi.fn()} />);
  await user.click(screen.getByRole('button', { name: 'Find installation' }));
  await user.click(screen.getByRole('button', { name: 'Open Codex login' }));
  vi.mocked(agents.onboarding).mockResolvedValue(ready);
  expect(
    await screen.findByText(
      'Authentication: authenticated',
      {},
      { timeout: 3000 },
    ),
  ).toBeTruthy();
  expect(agents.login).toHaveBeenCalledOnce();
  expect(agents.start).not.toHaveBeenCalled();
});
