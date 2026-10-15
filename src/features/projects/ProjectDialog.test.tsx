import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ProjectDialog } from './ProjectDialog';
import { api } from '../../lib/ipc';
import { useAppStore } from '../../app/store';
vi.mock('../../lib/ipc', async (original) => ({
  ...(await original<typeof import('../../lib/ipc')>()),
  api: { saveProject: vi.fn() },
}));
beforeEach(() =>
  useAppStore.setState({ busy: false, loading: false, error: null }),
);
afterEach(cleanup);
it('submits the complete project input and closes only after success', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  vi.mocked(api.saveProject).mockResolvedValue({
    workspaces: [],
    activeWorkspaceId: 'workspace',
    projects: [],
  });
  render(<ProjectDialog workspaceId="workspace" onClose={close} />);
  await user.type(screen.getByLabelText('Project name'), 'Compiler');
  await user.type(screen.getByLabelText(/Description/), 'Build a compiler');
  await user.selectOptions(screen.getByLabelText('Status'), 'active');
  await user.click(screen.getByRole('button', { name: 'rocket' }));
  await user.click(screen.getByRole('button', { name: '#10b981' }));
  await user.click(screen.getByRole('button', { name: 'Create project' }));
  expect(api.saveProject).toHaveBeenCalledWith(null, {
    workspaceId: 'workspace',
    name: 'Compiler',
    description: 'Build a compiler',
    icon: 'rocket',
    color: '#10b981',
    status: 'active',
  });
  expect(close).toHaveBeenCalledOnce();
});
it('keeps unsaved input visible when the backend rejects a project', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  vi.mocked(api.saveProject).mockRejectedValue({
    code: 'database',
    message: 'Could not save',
  });
  render(<ProjectDialog workspaceId="workspace" onClose={close} />);
  await user.type(screen.getByLabelText('Project name'), 'Compiler');
  await user.click(screen.getByRole('button', { name: 'Create project' }));
  expect(screen.getByRole('alert').textContent).toBe('Could not save');
  expect(
    (screen.getByLabelText('Project name') as HTMLInputElement).value,
  ).toBe('Compiler');
  expect(close).not.toHaveBeenCalled();
});
