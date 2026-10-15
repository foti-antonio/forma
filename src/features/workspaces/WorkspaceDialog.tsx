import { useState } from 'react';
import { Dialog } from '../../components/ui/dialog';
import { Button } from '../../components/ui/button';
import { useAppStore } from '../../app/store';
import { api } from '../../lib/ipc';
import type { Workspace } from '../../lib/ipc';
export function WorkspaceDialog({
  workspace,
  onClose,
}: {
  workspace?: Workspace;
  onClose: () => void;
}) {
  const [name, setName] = useState(workspace?.name ?? '');
  const { busy, error, mutate } = useAppStore();
  return (
    <Dialog
      title={workspace ? 'Rename workspace' : 'Create a workspace'}
      description="A dedicated home for your projects. Everything stays on this device."
      onClose={onClose}
      busy={busy}
    >
      <form
        onSubmit={async (event) => {
          event.preventDefault();
          if (
            await mutate(() =>
              workspace
                ? api.renameWorkspace(workspace.id, name)
                : api.createWorkspace(name),
            )
          )
            onClose();
        }}
      >
        <label htmlFor="workspace-name">Workspace name</label>
        <input
          id="workspace-name"
          autoFocus
          required
          maxLength={80}
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. Acme Studio"
          disabled={busy}
        />
        {error && (
          <p role="alert" className="form-error">
            {error}
          </p>
        )}
        <div className="dialog-actions">
          <Button
            type="button"
            variant="secondary"
            disabled={busy}
            onClick={onClose}
          >
            Cancel
          </Button>
          <Button disabled={busy || !name.trim()}>
            {busy ? 'Saving…' : workspace ? 'Save changes' : 'Create workspace'}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
