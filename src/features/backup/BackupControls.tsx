import { Dialog } from '../../components/ui/dialog';
import { Button } from '../../components/ui/button';
import { useAppStore } from '../../app/store';
import { useIssueStore } from '../issues/store';
import { useBackupStore } from './store';

export function BackupControls({ onRestored }: { onRestored: () => void }) {
  const state = useBackupStore();
  const appBusy = useAppStore((s) => s.busy || s.loading);
  const issueBusy = useIssueStore((s) => s.busy || s.loading);
  const disabled = state.busy || appBusy || issueBusy;
  return (
    <div className="backup-actions">
      <Button
        variant="ghost"
        disabled={disabled}
        onClick={() => void state.export()}
      >
        Export backup
      </Button>
      <Button
        variant="ghost"
        disabled={disabled}
        onClick={() => void state.prepare()}
      >
        Restore backup…
      </Button>
      {state.busy && <p role="status">Working with local backup…</p>}
      {state.error && (
        <p role="alert" className="form-error">
          {state.error}
        </p>
      )}
      {state.message && <p role="status">{state.message}</p>}
      {state.prepared && (
        <Dialog
          title="Replace all local data?"
          description="The selected backup passed validation. Restoring replaces every workspace, project, issue, label, relationship, activity entry and saved preference. Forma first saves a recovery copy of your current data. Backups are not encrypted."
          busy={state.busy}
          onClose={() => void state.cancel()}
        >
          <div className="dialog-actions">
            <Button
              variant="secondary"
              disabled={state.busy}
              onClick={() => void state.cancel()}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              disabled={state.busy}
              onClick={async () => {
                if (await state.restore()) onRestored();
              }}
            >
              Replace local data
            </Button>
          </div>
        </Dialog>
      )}
    </div>
  );
}
