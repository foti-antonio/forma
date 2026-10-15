import { useState } from 'react';
import { Dialog } from '../../components/ui/dialog';
import { Button } from '../../components/ui/button';
import type { Label } from '../../lib/ipc';
import { useIssueStore } from './store';
export function LabelsDialog({
  workspaceId,
  onClose,
}: {
  workspaceId: string;
  onClose: () => void;
}) {
  const {
    labels,
    loading,
    error,
    refresh,
    busy,
    mutationError,
    saveLabel,
    deleteLabel,
    clearError,
  } = useIssueStore();
  const [editing, setEditing] = useState<Label | null>(null);
  const [name, setName] = useState('');
  const [color, setColor] = useState('#8b5cf6');
  const [deleting, setDeleting] = useState<Label | null>(null);
  const reset = () => {
    setEditing(null);
    setName('');
    setColor('#8b5cf6');
    clearError();
  };
  return (
    <Dialog
      title="Workspace labels"
      description="Use a shared vocabulary across projects in this workspace."
      busy={busy}
      onClose={onClose}
    >
      {error && (
        <div role="alert" className="form-error">
          {error}
          <Button
            variant="secondary"
            disabled={loading}
            onClick={() => void refresh()}
          >
            Retry labels
          </Button>
        </div>
      )}
      {loading && <p role="status">Loading labels…</p>}
      <div className="label-list">
        {labels.map((l) => (
          <div className="label-row" key={l.id}>
            <span className="label-dot" style={{ background: l.color }} />
            <span>{l.name}</span>
            <Button
              variant="ghost"
              disabled={busy}
              onClick={() => {
                clearError();
                setEditing(l);
                setName(l.name);
                setColor(l.color);
                setDeleting(null);
              }}
            >
              Edit {l.name}
            </Button>
            <Button
              variant="ghost"
              disabled={busy}
              onClick={() => {
                clearError();
                setDeleting(l);
              }}
            >
              Delete {l.name}
            </Button>
          </div>
        ))}
        {!labels.length && (
          <p className="muted">No labels yet. Add your first one below.</p>
        )}
      </div>
      <form
        onSubmit={async (e) => {
          e.preventDefault();
          if (
            await saveLabel(editing?.id ?? null, { workspaceId, name, color })
          )
            reset();
        }}
      >
        <fieldset disabled={busy} className="form-fields">
          <label htmlFor="label-name">Label name</label>
          <input
            id="label-name"
            autoFocus
            required
            maxLength={40}
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. Bug, Design, Improvement"
          />
          <label htmlFor="label-color">Label color</label>
          <input
            id="label-color"
            type="color"
            value={color}
            onChange={(e) => setColor(e.target.value)}
          />
        </fieldset>
        <div className="dialog-actions">
          {editing && (
            <Button
              type="button"
              variant="secondary"
              disabled={busy}
              onClick={reset}
            >
              Cancel edit
            </Button>
          )}
          <Button disabled={busy || !name.trim()}>
            {busy ? 'Saving…' : editing ? 'Save label' : 'Create label'}
          </Button>
        </div>
      </form>
      {deleting && (
        <div
          className="inline-confirm"
          role="alertdialog"
          aria-label="Confirm label deletion"
        >
          <p>
            Delete “{deleting.name}”? This removes it from all issues in this
            workspace.
          </p>
          <Button
            variant="secondary"
            autoFocus
            disabled={busy}
            onClick={() => setDeleting(null)}
          >
            Keep label
          </Button>
          <Button
            variant="destructive"
            disabled={busy}
            onClick={async () => {
              if (await deleteLabel(deleting.id)) {
                setDeleting(null);
                if (editing?.id === deleting.id) reset();
              }
            }}
          >
            Delete permanently
          </Button>
        </div>
      )}
      {mutationError && (
        <p role="alert" className="form-error">
          {mutationError}
        </p>
      )}
    </Dialog>
  );
}
