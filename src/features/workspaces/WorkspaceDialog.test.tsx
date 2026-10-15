import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { WorkspaceDialog } from './WorkspaceDialog';
import { ConfirmDialog } from '../../components/ConfirmDialog';
import { api } from '../../lib/ipc';
import { useAppStore } from '../../app/store';
vi.mock('../../lib/ipc', async (original) => ({
  ...(await original<typeof import('../../lib/ipc')>()),
  api: { createWorkspace: vi.fn(), renameWorkspace: vi.fn() },
}));
const snapshot = { workspaces: [], activeWorkspaceId: null, projects: [] };
beforeEach(() =>
  useAppStore.setState({ busy: false, loading: false, error: null }),
);
afterEach(cleanup);
it('creates a workspace using the backend snapshot', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  vi.mocked(api.createWorkspace).mockResolvedValue(snapshot);
  render(<WorkspaceDialog onClose={close} />);
  await user.type(screen.getByLabelText('Workspace name'), 'Studio');
  await user.click(screen.getByRole('button', { name: 'Create workspace' }));
  expect(api.createWorkspace).toHaveBeenCalledWith('Studio');
  expect(close).toHaveBeenCalledOnce();
});
it('renames the existing workspace and keeps the same identifier', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  vi.mocked(api.renameWorkspace).mockResolvedValue(snapshot);
  render(
    <WorkspaceDialog
      workspace={{
        id: 'workspace',
        name: 'Old name',
        createdAt: '',
        updatedAt: '',
      }}
      onClose={close}
    />,
  );
  await user.clear(screen.getByLabelText('Workspace name'));
  await user.type(screen.getByLabelText('Workspace name'), 'New name');
  await user.click(screen.getByRole('button', { name: 'Save changes' }));
  expect(api.renameWorkspace).toHaveBeenCalledWith('workspace', 'New name');
  expect(close).toHaveBeenCalledOnce();
});
it('does not delete anything when confirmation is cancelled', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  const action = vi.fn();
  render(
    <ConfirmDialog
      title="Delete workspace?"
      description="All projects will be deleted."
      action={action}
      onClose={close}
    />,
  );
  expect(action).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Cancel' }));
  expect(action).not.toHaveBeenCalled();
  expect(close).toHaveBeenCalledOnce();
});
it('calls the destructive action only after explicit confirmation', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  const action = vi.fn().mockResolvedValue(snapshot);
  render(
    <ConfirmDialog
      title="Delete project?"
      description="This cannot be undone."
      action={action}
      onClose={close}
    />,
  );
  expect(action).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Delete permanently' }));
  expect(action).toHaveBeenCalledOnce();
  expect(close).toHaveBeenCalledOnce();
});
