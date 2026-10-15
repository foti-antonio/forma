import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BackupControls } from './BackupControls';
import { useBackupStore } from './store';
import { useAppStore } from '../../app/store';
import { useIssueStore } from '../issues/store';
import { api } from '../../lib/ipc';
vi.mock('../../lib/ipc', async (original) => ({
  ...(await original<typeof import('../../lib/ipc')>()),
  api: {
    exportBackup: vi.fn(),
    prepareRestore: vi.fn(),
    restoreBackup: vi.fn(),
    cancelRestore: vi.fn(),
    snapshot: vi.fn(),
  },
}));
beforeEach(() => {
  vi.resetAllMocks();
  useBackupStore.setState({
    busy: false,
    prepared: false,
    error: null,
    message: '',
  });
  useAppStore.setState({ busy: false, loading: false });
  useIssueStore.setState({ busy: false, loading: false });
  vi.mocked(api.prepareRestore).mockResolvedValue(true);
  vi.mocked(api.restoreBackup).mockResolvedValue(
    '/app-data/recovery.forma-backup',
  );
  vi.mocked(api.snapshot).mockResolvedValue({
    workspaces: [],
    projects: [],
    activeWorkspaceId: null,
    theme: 'dark',
  });
});
afterEach(cleanup);
it('validates a native selection then requires explicit confirmation and refreshes all state', async () => {
  const user = userEvent.setup(),
    done = vi.fn();
  render(<BackupControls onRestored={done} />);
  await user.click(screen.getByRole('button', { name: 'Restore backup…' }));
  await screen.findByRole('dialog', { name: 'Replace all local data?' });
  expect(api.restoreBackup).not.toHaveBeenCalled();
  expect(screen.getByText(/Backups are not encrypted/)).toBeTruthy();
  await user.click(screen.getByRole('button', { name: 'Replace local data' }));
  await waitFor(() => expect(done).toHaveBeenCalledOnce());
  expect(api.snapshot).toHaveBeenCalledOnce();
  expect(useIssueStore.getState().query).toBeNull();
  expect(screen.getByRole('status').textContent).toContain(
    '/app-data/recovery.forma-backup',
  );
});
it('cancels staged restore and handles invalid backups and failed replacement', async () => {
  const user = userEvent.setup();
  render(<BackupControls onRestored={vi.fn()} />);
  await user.click(screen.getByRole('button', { name: 'Restore backup…' }));
  await screen.findByRole('dialog');
  await user.click(screen.getByRole('button', { name: 'Cancel' }));
  expect(api.cancelRestore).toHaveBeenCalledOnce();
  expect(api.restoreBackup).not.toHaveBeenCalled();
  vi.mocked(api.prepareRestore).mockRejectedValueOnce(
    new Error('Invalid backup'),
  );
  await user.click(screen.getByRole('button', { name: 'Restore backup…' }));
  await waitFor(() =>
    expect(screen.getByRole('alert').textContent).toBe('Invalid backup'),
  );
  vi.mocked(api.restoreBackup).mockRejectedValueOnce(
    new Error('Recovery copy failed'),
  );
  await user.click(screen.getByRole('button', { name: 'Restore backup…' }));
  await screen.findByRole('dialog');
  await user.click(screen.getByRole('button', { name: 'Replace local data' }));
  await waitFor(() =>
    expect(screen.getByRole('alert').textContent).toBe('Recovery copy failed'),
  );
  expect(api.snapshot).not.toHaveBeenCalled();
});
it('handles native dialog cancellation, export errors and successful unencrypted export', async () => {
  const user = userEvent.setup();
  render(<BackupControls onRestored={vi.fn()} />);
  vi.mocked(api.exportBackup).mockResolvedValueOnce(false);
  await user.click(screen.getByRole('button', { name: 'Export backup' }));
  expect(screen.queryByRole('status')).toBeNull();
  vi.mocked(api.exportBackup).mockRejectedValueOnce(new Error('No space'));
  await user.click(screen.getByRole('button', { name: 'Export backup' }));
  expect(screen.getByRole('alert').textContent).toBe('No space');
  vi.mocked(api.exportBackup).mockResolvedValueOnce(true);
  await user.click(screen.getByRole('button', { name: 'Export backup' }));
  expect(screen.getByRole('status').textContent).toContain('not encrypted');
});
