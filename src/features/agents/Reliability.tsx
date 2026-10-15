import { useEffect, useState } from 'react';
import { Button } from '../../components/ui/button';
import { Dialog } from '../../components/ui/dialog';
import {
  errorMessage,
  type IndependentTest,
  type TestCommand,
  type TestPreview,
  type WorktreeInfo,
} from '../../lib/ipc';
import { agents, isActive } from './store';

export function Worktrees() {
  const [items, setItems] = useState<WorktreeInfo[]>([]);
  const [offset, setOffset] = useState(0);
  const [revision, setRevision] = useState(0);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [confirm, setConfirm] = useState<WorktreeInfo | null>(null);
  useEffect(() => {
    let stale = false;
    setLoading(true);
    agents
      .worktrees(offset)
      .then((v) => {
        if (!stale) {
          setItems(v);
          setError('');
        }
      })
      .catch((e) => {
        if (!stale) setError(errorMessage(e));
      })
      .finally(() => {
        if (!stale) setLoading(false);
      });
    return () => {
      stale = true;
    };
  }, [offset, revision]);
  const perform = async (action: () => Promise<unknown>) => {
    setBusy(true);
    setError('');
    try {
      await action();
      setRevision((v) => v + 1);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <section aria-label="Worktree Management" aria-busy={loading}>
      <h3>Worktree Management</h3>
      <p>
        Runs retain their branches and worktrees for review. Cleanup preserves
        the branch and historical run metadata. Modified or unreviewed worktrees
        require manual preservation before cleanup.
      </p>
      {error && <p role="alert">{error}</p>}
      {loading && <p role="status">Inspecting worktrees…</p>}
      {!loading && !items.length && <p>No worktrees on this page.</p>}
      <Button
        disabled={busy || loading}
        onClick={() => setRevision((v) => v + 1)}
      >
        Refresh worktrees
      </Button>
      {items.map((item) => (
        <article className="agent-run" key={item.run.id}>
          <strong>
            {item.run.issueIdentifier} · {item.run.issueTitle}
          </strong>
          <p>
            {item.run.status} · {item.cleanupState} · Review:{' '}
            {item.run.review ?? 'pending'} ·{' '}
            {item.dirty === null
              ? 'Changes unknown'
              : item.dirty
                ? 'Local changes present'
                : 'Clean'}
          </p>
          <p className="git-target">
            Run: {item.run.id}
            <br />
            Repository: {item.repository}
            <br />
            Branch: {item.run.branch}
            <br />
            Path: {item.run.worktree}
          </p>
          <p>
            Disk usage:{' '}
            {item.diskBytes === null
              ? 'Unavailable (missing, inaccessible, symlinked or over 10,000 entries)'
              : `${item.diskBytes.toLocaleString()} bytes`}
          </p>
          <p>{item.message}</p>
          <Button
            disabled={
              busy || !item.run.trusted || item.cleanupState === 'Removed'
            }
            variant="secondary"
            onClick={() => void perform(() => agents.open(item.run.id))}
          >
            Open worktree
          </Button>
          <Button
            disabled={
              busy ||
              isActive(item.run.status) ||
              !item.run.review ||
              item.dirty !== false ||
              item.cleanupState === 'Removed'
            }
            variant="destructive"
            onClick={() => setConfirm(item)}
          >
            Clean up reviewed worktree…
          </Button>
        </article>
      ))}
      <div className="dialog-actions">
        <Button
          disabled={loading || offset === 0}
          onClick={() => setOffset((v) => Math.max(0, v - 20))}
        >
          Previous worktrees
        </Button>
        <Button
          disabled={loading || items.length < 20 || offset >= 10000}
          onClick={() => setOffset((v) => v + 20)}
        >
          Next worktrees
        </Button>
      </div>
      {confirm && (
        <Dialog
          title="Delete reviewed worktree?"
          description="This removes the clean linked worktree folder. The branch and historical run remain. Forma checks the path, repository, changes and review again before deleting."
          onClose={() => setConfirm(null)}
          busy={busy}
        >
          <p className="git-target">{confirm.run.worktree}</p>
          {error && <p role="alert">{error}</p>}
          <Button
            disabled={busy}
            variant="secondary"
            onClick={() => setConfirm(null)}
          >
            Keep worktree
          </Button>
          <Button
            disabled={busy}
            variant="destructive"
            onClick={() =>
              void perform(async () => {
                await agents.cleanup(confirm.run.id);
                setConfirm(null);
              })
            }
          >
            Delete clean worktree
          </Button>
        </Dialog>
      )}
    </section>
  );
}

export function IndependentTests({ runId }: { runId: string }) {
  const [project, setProject] = useState('');
  const [command, setCommand] = useState<TestCommand | null>(null);
  const [args, setArgs] = useState('[]');
  const [tests, setTests] = useState<IndependentTest[]>([]);
  const [preview, setPreview] = useState<TestPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    let stale = false;
    agents
      .testProject(runId)
      .then(async (p) => {
        const c = await agents.testCommand(p);
        if (!stale) {
          setProject(p);
          setCommand(c);
          setArgs(JSON.stringify(c?.arguments ?? []));
        }
      })
      .catch((e) => {
        if (!stale) setError(errorMessage(e));
      })
      .finally(() => {
        if (!stale) setLoading(false);
      });
    return () => {
      stale = true;
    };
  }, [runId, revision]);
  useEffect(() => {
    let stale = false;
    let timer: ReturnType<typeof setTimeout>;
    const load = async () => {
      try {
        const v = await agents.testRuns(runId);
        if (!stale) setTests(v);
      } catch (e) {
        if (!stale) setError(errorMessage(e));
      } finally {
        if (!stale) timer = setTimeout(() => void load(), 1000);
      }
    };
    void load();
    return () => {
      stale = true;
      clearTimeout(timer);
    };
  }, [runId]);
  const perform = async (action: () => Promise<unknown>) => {
    setBusy(true);
    setError('');
    try {
      await action();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <section aria-label="Independent tests">
      <h4>Independent tests</h4>
      <p>
        Configure a trusted executable and JSON argument array for this project.
        Repository test code runs with your OS privileges and may access files,
        network and credentials. Review the command and working directory before
        each execution.
      </p>
      {loading && <p role="status">Loading test configuration…</p>}
      {error && <p role="alert">{error}</p>}
      <Button
        disabled={busy || loading || !project}
        variant="secondary"
        onClick={() =>
          void perform(async () => {
            await agents.testAuthorize(project);
            setRevision((v) => v + 1);
          })
        }
      >
        Choose & authorize test executable…
      </Button>
      {command && (
        <fieldset disabled={busy}>
          <legend>Project test command</legend>
          <p className="git-target">
            {command.executable} ·{' '}
            {command.trusted ? 'Authorized' : 'Reauthorization required'}
          </p>
          <label htmlFor="test-args">
            Arguments (JSON array, no shell expansion)
          </label>
          <textarea
            id="test-args"
            value={args}
            onChange={(e) => setArgs(e.target.value)}
            rows={3}
          />
          <label htmlFor="test-runtime">Test timeout (seconds)</label>
          <input
            id="test-runtime"
            type="number"
            min={1}
            max={7200}
            value={command.maxRuntime}
            onChange={(e) =>
              setCommand({ ...command, maxRuntime: Number(e.target.value) })
            }
          />
          <label htmlFor="test-output">Test output limit (bytes)</label>
          <input
            id="test-output"
            type="number"
            min={1024}
            max={1048576}
            value={command.logLimit}
            onChange={(e) =>
              setCommand({ ...command, logLimit: Number(e.target.value) })
            }
          />
          <Button
            disabled={!command.trusted}
            onClick={() =>
              void perform(async () => {
                const value: unknown = JSON.parse(args);
                if (
                  !Array.isArray(value) ||
                  !value.every((a) => typeof a === 'string')
                )
                  throw new Error('Use a JSON array of strings.');
                await agents.testSaveCommand({ ...command, arguments: value });
                setPreview(await agents.testPreview(runId));
              })
            }
          >
            Save & preview independent tests
          </Button>
        </fieldset>
      )}
      {preview && (
        <div
          className="inline-confirm"
          role="group"
          aria-label="Approve independent tests"
        >
          <strong>Approve independent test execution?</strong>
          <pre className="agent-output">
            {JSON.stringify(
              [preview.command.executable, ...preview.command.arguments],
              null,
              2,
            )}
          </pre>
          <p className="git-target">
            Working directory: {preview.workingDirectory}
          </p>
          <p>
            {preview.command.maxRuntime}s timeout · {preview.command.logLimit}{' '}
            output bytes · Approval expires in five minutes.
          </p>
          <Button
            disabled={busy}
            variant="secondary"
            onClick={() => setPreview(null)}
          >
            Cancel test approval
          </Button>
          <Button
            disabled={busy}
            onClick={() =>
              void perform(async () => {
                await agents.testStart(preview.token);
                setPreview(null);
              })
            }
          >
            Approve & run independent tests
          </Button>
        </div>
      )}
      {!tests.length && <p>No independently executed tests recorded.</p>}
      {tests.map((test) => (
        <article key={test.id}>
          <strong>
            Independent test · {test.status} · Exit {test.exitCode ?? '—'}
          </strong>
          <p>
            {test.durationMs === null
              ? 'Duration pending'
              : `${test.durationMs} ms`}{' '}
            · {test.result}
          </p>
          <pre className="agent-output">
            {JSON.stringify([test.executable, ...test.arguments])}
            {'\n'}
            {test.workingDirectory}
            {'\n'}
            {test.log || 'No output received.'}
          </pre>
          {test.status === 'Running' && (
            <Button
              disabled={busy}
              variant="destructive"
              onClick={() => void perform(() => agents.testCancel(test.id))}
            >
              Cancel independent tests
            </Button>
          )}
        </article>
      ))}
    </section>
  );
}
