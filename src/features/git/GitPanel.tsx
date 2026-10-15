import { useEffect, useState } from 'react';
import { Button } from '../../components/ui/button';
import { Dialog } from '../../components/ui/dialog';
import type { GitCommit } from '../../lib/ipc';
import { useGitStore } from './store';

export function CommitDetails({ commit }: { commit: GitCommit }) {
  return (
    <div className="git-commit">
      <code title={commit.hash}>{commit.hash.slice(0, 12)}</code>
      <p className="git-message">{commit.message || '(No commit message)'}</p>
      <span className="muted">
        {commit.author || 'Unknown author'} ·{' '}
        <time dateTime={commit.committedAt}>{commit.committedAt}</time>
      </span>
    </div>
  );
}
export function GitPanel({ projectId }: { projectId: string }) {
  const {
    repository,
    overview,
    branches,
    loading,
    busy,
    error,
    load,
    connect,
    disconnect,
  } = useGitStore();
  const [search, setSearch] = useState('');
  const [branchOffset, setBranchOffset] = useState(0);
  const [commitOffset, setCommitOffset] = useState(0);
  const [confirm, setConfirm] = useState(false);
  useEffect(() => {
    void load(projectId, search, branchOffset, commitOffset);
  }, [projectId, search, branchOffset, commitOffset, load]);
  const refresh = () =>
    void load(projectId, search, branchOffset, commitOffset);
  return (
    <section
      className="git-panel"
      aria-label="Project Git"
      aria-busy={loading || busy}
    >
      <div className="section-heading">
        <h2>Git</h2>
        <div className="issue-tools">
          <Button
            variant="secondary"
            disabled={loading || busy}
            onClick={async () => {
              await connect(projectId);
              setBranchOffset(0);
              setCommitOffset(0);
            }}
          >
            {' '}
            {repository ? 'Reconnect folder…' : 'Connect repository…'}
          </Button>
          <Button variant="ghost" disabled={loading || busy} onClick={refresh}>
            Refresh Git
          </Button>
          {repository && (
            <Button
              variant="ghost"
              disabled={loading || busy}
              onClick={() => setConfirm(true)}
            >
              Disconnect
            </Button>
          )}
        </div>
      </div>
      <p className="muted small-text">
        One local repository per project. Git inspection is read-only; refresh
        manually.
      </p>
      {loading && <p role="status">Reading Git…</p>}
      {error && (
        <p role="alert" className="form-error">
          {error}
        </p>
      )}
      {!loading && !repository && !error && (
        <p className="muted">
          Connect an existing repository to inspect branches, changes and
          commits.
        </p>
      )}
      {repository && (
        <div>
          <strong>{repository.name}</strong>
          <p className="git-path">{repository.path}</p>
          {!repository.trusted && (
            <p role="status">
              Restored connection. Select the folder again to allow local
              access.
            </p>
          )}
        </div>
      )}
      {overview && (
        <>
          <p>
            <strong>{overview.branch}</strong> · Repository readable
          </p>
          {overview.status ? (
            <p>
              {overview.status.modified} modified · {overview.status.added}{' '}
              added · {overview.status.deleted} deleted ·{' '}
              {overview.status.untracked} untracked ·{' '}
              {overview.status.conflicted} conflicted
            </p>
          ) : (
            <p className="muted">
              Bare repository · working-tree status unavailable.
            </p>
          )}
          <div className="git-columns">
            <section aria-label="Local branches">
              <h3>Local branches</h3>
              <label htmlFor="git-branch-search">Find branches</label>
              <input
                id="git-branch-search"
                type="search"
                maxLength={200}
                value={search}
                onChange={(e) => {
                  setSearch(e.target.value);
                  setBranchOffset(0);
                }}
              />
              {branches?.branches.map((b) => (
                <div className="activity-row" key={b.name}>
                  <div>
                    <strong className="git-target">{b.name}</strong>
                    {b.active && <span> · Active</span>}
                    <CommitDetails commit={b.commit} />
                  </div>
                </div>
              ))}
              {!branches?.branches.length && (
                <p className="muted">No local branches match.</p>
              )}
              {branches?.truncated && (
                <p role="status">
                  Stopped after 10,000 branches. Refine the search or inspect
                  the repository externally.
                </p>
              )}
              <div className="pagination">
                <Button
                  variant="ghost"
                  disabled={loading || !branchOffset}
                  onClick={() => setBranchOffset((v) => v - 25)}
                >
                  Previous branches
                </Button>
                <Button
                  variant="ghost"
                  disabled={
                    loading || !branches?.hasMore || branchOffset >= 10000
                  }
                  onClick={() => setBranchOffset((v) => v + 25)}
                >
                  Next branches
                </Button>
              </div>
            </section>
            <section aria-label="Recent commits">
              <h3>Recent commits</h3>
              {overview.commits.map((c) => (
                <div className="activity-row" key={c.hash}>
                  <CommitDetails commit={c} />
                </div>
              ))}
              {!overview.commits.length && (
                <p className="muted">No commits yet.</p>
              )}
              <div className="pagination">
                <Button
                  variant="ghost"
                  disabled={loading || !commitOffset}
                  onClick={() => setCommitOffset((v) => v - 20)}
                >
                  Newer commits
                </Button>
                <Button
                  variant="ghost"
                  disabled={
                    loading || !overview.hasMore || commitOffset >= 10000
                  }
                  onClick={() => setCommitOffset((v) => v + 20)}
                >
                  Older commits
                </Button>
              </div>
            </section>
          </div>
        </>
      )}
      {confirm && (
        <Dialog
          title="Disconnect repository?"
          description="Saved issue links remain as historical records. Repository files are untouched."
          busy={busy}
          onClose={() => setConfirm(false)}
        >
          <div className="dialog-actions">
            <Button
              variant="secondary"
              disabled={busy}
              onClick={() => setConfirm(false)}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              disabled={busy}
              onClick={async () => {
                await disconnect(projectId);
                setConfirm(false);
              }}
            >
              Disconnect repository
            </Button>
          </div>
        </Dialog>
      )}
    </section>
  );
}
