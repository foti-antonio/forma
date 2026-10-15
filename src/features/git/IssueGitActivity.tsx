import { useEffect, useState } from 'react';
import { Button } from '../../components/ui/button';
import { errorMessage } from '../../lib/ipc';
import type {
  Issue,
  GitLink,
  GitBranch,
  GitCommit,
  GitRepository,
} from '../../lib/ipc';
import { CommitDetails } from './GitPanel';
import { gitActivity, mentionsIssue } from './store';

export function IssueGitActivity({
  issue,
  disabled,
}: {
  issue: Issue;
  disabled: boolean;
}) {
  const [open, setOpen] = useState(false);
  return (
    <details
      className="issue-related"
      open={open}
      onToggle={(e) => setOpen(e.currentTarget.open)}
    >
      <summary>Git activity</summary>
      {open && <Activity issue={issue} disabled={disabled} />}
    </details>
  );
}
function Activity({ issue, disabled }: { issue: Issue; disabled: boolean }) {
  const [links, setLinks] = useState<GitLink[]>([]);
  const [repository, setRepository] = useState<GitRepository | null>(null);
  const [branches, setBranches] = useState<GitBranch[]>([]);
  const [commits, setCommits] = useState<GitCommit[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);
  const [kind, setKind] = useState<'branch' | 'commit'>('branch');
  const [target, setTarget] = useState('');
  const [pending, setPending] = useState<{
    kind: 'branch' | 'commit';
    target: string;
  } | null>(null);
  const [remove, setRemove] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    const read = async () => {
      // History remains available even if local Git inspection fails.
      const links = await gitActivity.links(issue.projectId, issue.id);
      if (cancelled) return;
      setLinks(links);
      const repository = await gitActivity.connection(issue.projectId);
      if (cancelled) return;
      setRepository(repository);
      setBranches([]);
      setCommits([]);
      if (repository?.trusted) {
        const [branches, overview] = await Promise.all([
          gitActivity.branches(issue.projectId, repository.id),
          gitActivity.overview(issue.projectId, repository.id),
        ]);
        if (!cancelled) {
          setBranches(branches.branches);
          setCommits(overview.commits);
        }
      }
    };
    void read()
      .catch((e) => {
        if (!cancelled) setError(errorMessage(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [issue.id, issue.projectId, revision]);
  const locked = disabled || busy || loading;
  const save = async () => {
    if (!pending || !repository || locked) return;
    setBusy(true);
    setError(null);
    try {
      setLinks(
        await gitActivity.save({
          projectId: issue.projectId,
          issueId: issue.id,
          repositoryId: repository.id,
          kind: pending.kind,
          target: pending.target,
          confirmed: true,
        }),
      );
      setPending(null);
      setTarget('');
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <section aria-label="Issue Git activity" aria-busy={busy || loading}>
      <div className="section-heading">
        <h3>Git activity</h3>
        <Button
          type="button"
          variant="ghost"
          disabled={locked}
          onClick={() => setRevision((v) => v + 1)}
        >
          Refresh Git activity
        </Button>
      </div>
      {loading && <p role="status">Loading Git activity…</p>}
      {error && (
        <p role="alert" className="form-error">
          {error}
        </p>
      )}
      {!loading && !links.length && (
        <p className="muted">No Git work linked to this issue.</p>
      )}
      {links.map((link) => (
        <div className="activity-row" key={link.id}>
          <div>
            <strong className="git-target">
              {link.kind}: {link.target}
            </strong>
            <p className="muted">
              {link.repositoryName}
              {!link.connected && ' · Disconnected'} · Saved snapshot
            </p>
            <CommitDetails commit={link.commit} />
            <p className="muted small-text">
              Branch names are historical; deleted or renamed branches keep
              their original link.
            </p>
            <Button
              type="button"
              variant="ghost"
              disabled={locked}
              onClick={() => setRemove(link.id)}
            >
              Remove Git link
            </Button>
          </div>
        </div>
      ))}
      {remove && (
        <section
          role="alertdialog"
          aria-label="Remove Git association?"
          className="inline-confirm"
        >
          <p>Remove this association? Git files remain untouched.</p>
          <Button
            type="button"
            variant="secondary"
            disabled={busy}
            onClick={() => setRemove(null)}
          >
            Keep link
          </Button>
          <Button
            type="button"
            variant="destructive"
            disabled={locked}
            onClick={async () => {
              setBusy(true);
              try {
                setLinks(
                  await gitActivity.remove(issue.projectId, issue.id, remove),
                );
                setRemove(null);
              } catch (e) {
                setError(errorMessage(e));
              } finally {
                setBusy(false);
              }
            }}
          >
            Confirm removal
          </Button>
        </section>
      )}
      {repository?.trusted ? (
        <>
          <p className="muted small-text">
            Suggestions use exact {issue.identifier} tokens in the first 25
            branches and 20 recent commits. Review every association before
            saving.
          </p>
          {branches
            .filter((b) => mentionsIssue(b.name, issue.identifier))
            .map((b) => (
              <Button
                type="button"
                variant="secondary"
                key={b.name}
                disabled={locked}
                onClick={() => setPending({ kind: 'branch', target: b.name })}
              >
                Suggested branch: {b.name}
              </Button>
            ))}
          {commits
            .filter((c) => mentionsIssue(c.message, issue.identifier))
            .map((c) => (
              <Button
                type="button"
                variant="secondary"
                key={c.hash}
                disabled={locked}
                onClick={() => setPending({ kind: 'commit', target: c.hash })}
              >
                Suggested commit: {c.hash.slice(0, 12)}
              </Button>
            ))}
          <div className="form-fields">
            <label htmlFor="git-link-kind">Git work type</label>
            <select
              id="git-link-kind"
              disabled={locked}
              value={kind}
              onChange={(e) => {
                setKind(e.target.value as 'branch' | 'commit');
                setTarget('');
                setPending(null);
              }}
            >
              <option value="branch">Local branch</option>
              <option value="commit">Commit</option>
            </select>
            <label htmlFor="git-link-target">
              {kind === 'branch' ? 'Exact branch name' : 'Full commit hash'}
            </label>
            <input
              id="git-link-target"
              disabled={locked}
              maxLength={1024}
              value={target}
              onChange={(e) => setTarget(e.target.value)}
              placeholder={
                kind === 'branch'
                  ? 'feature/F1-1-work'
                  : '40-character commit hash'
              }
            />
            <Button
              type="button"
              variant="secondary"
              disabled={locked || !target.trim()}
              onClick={() => setPending({ kind, target: target.trim() })}
            >
              Review Git link
            </Button>
          </div>
          {pending && (
            <section
              className="inline-confirm"
              role="alertdialog"
              aria-label="Confirm Git association"
            >
              <p>
                Link {pending.kind}{' '}
                <span className="git-target">{pending.target}</span> from{' '}
                {repository.name} to {issue.identifier}?
              </p>
              <Button
                type="button"
                variant="secondary"
                disabled={busy}
                onClick={() => setPending(null)}
              >
                Cancel link
              </Button>
              <Button
                type="button"
                disabled={locked}
                onClick={() => void save()}
              >
                Confirm Git link
              </Button>
            </section>
          )}
        </>
      ) : (
        !loading && (
          <p className="muted">
            Connect or reauthorize a repository in the project’s Git section to
            add links.
          </p>
        )
      )}
    </section>
  );
}
