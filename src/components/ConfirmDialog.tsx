import { Dialog } from './ui/dialog';
import { Button } from './ui/button';
import { useAppStore } from '../app/store';
import type { Snapshot } from '../lib/ipc';
export function ConfirmDialog({
  title,
  description,
  action,
  onClose,
}: {
  title: string;
  description: string;
  action: () => Promise<Snapshot>;
  onClose: () => void;
}) {
  const { busy, error, mutate } = useAppStore();
  return (
    <Dialog
      title={title}
      description={description}
      onClose={onClose}
      busy={busy}
    >
      {error && (
        <p role="alert" className="form-error">
          {error}
        </p>
      )}
      <div className="dialog-actions">
        <Button variant="secondary" disabled={busy} onClick={onClose}>
          Cancel
        </Button>
        <Button
          variant="destructive"
          disabled={busy}
          onClick={async () => {
            if (await mutate(action)) onClose();
          }}
        >
          {busy ? 'Deleting…' : 'Delete permanently'}
        </Button>
      </div>
    </Dialog>
  );
}
