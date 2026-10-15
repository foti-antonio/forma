import { useEffect, useState } from 'react';
import { Button } from '../../components/ui/button';
import { Dialog } from '../../components/ui/dialog';
import {
  errorMessage,
  type AgentSettings,
  type QueuedTask,
  type AgentPreview,
  type AgentRun,
  type AgentReview,
  type ProviderStatus,
  type Issue,
} from '../../lib/ipc';
import { Worktrees, IndependentTests } from './Reliability';
import { agents, isActive } from './store';

export function AgentHub() {
  const [open, setOpen] = useState(false);
  const [tab, setTab] = useState<'runs' | 'settings' | 'queue' | 'worktrees'>(
    'runs',
  );
  return (
    <>
      <Button type="button" variant="ghost" onClick={() => setOpen(true)}>
        AI agents & runs
      </Button>
      {open && (
        <Dialog
          title="AI agents"
          description="Local CLI execution requires approval for each task. Provider authentication stays with the CLI."
          onClose={() => setOpen(false)}
          className="issue-dialog"
        >
          <div role="group" aria-label="Agent views">
            <Button
              type="button"
              variant={tab === 'runs' ? 'default' : 'secondary'}
              onClick={() => setTab('runs')}
            >
              Agent runs
            </Button>
            <Button
              type="button"
              variant={tab === 'settings' ? 'default' : 'secondary'}
              onClick={() => setTab('settings')}
            >
              Agent settings
            </Button>
          </div>
          <Button
            type="button"
            variant="secondary"
            onClick={() => setTab('queue')}
          >
            Approved task queue
          </Button>
          <Button
            type="button"
            variant="secondary"
            onClick={() => setTab('worktrees')}
          >
            Worktree Management
          </Button>
          {tab === 'settings' ? (
            <Settings />
          ) : tab === 'worktrees' ? (
            <Worktrees />
          ) : tab === 'queue' ? (
            <Queue />
          ) : (
            <Runs />
          )}
        </Dialog>
      )}
    </>
  );
}
function Settings() {
  const [providers, setProviders] = useState<ProviderStatus[]>([]);
  const [preferred, setPreferred] = useState('codex');
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  useEffect(() => {
    let stale = false;
    Promise.all([agents.providers(), agents.preferred()])
      .then(([p, v]) => {
        if (!stale) {
          setProviders(p);
          setPreferred(v);
        }
      })
      .catch((e) => {
        if (!stale) setError(errorMessage(e));
      })
      .finally(() => {
        if (!stale) setBusy(false);
      });
    return () => {
      stale = true;
    };
  }, []);
  const perform = async (action: () => Promise<unknown>) => {
    setBusy(true);
    setError('');
    setMessage('');
    try {
      await action();
      setProviders(await agents.providers());
      setMessage('Agent settings updated.');
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };
  const edit = (provider: string, patch: Partial<AgentSettings>) =>
    setProviders((values) =>
      values.map((p) =>
        p.settings.provider === provider
          ? { ...p, settings: { ...p.settings, ...patch } }
          : p,
      ),
    );
  return (
    <section aria-label="Agent settings" aria-busy={busy}>
      {error && (
        <p role="alert" className="form-error">
          {error}
        </p>
      )}
      {message && <p role="status">{message}</p>}
      {busy && <p role="status">Inspecting agent settings…</p>}
      <label htmlFor="preferred-agent">Preferred agent</label>
      <select
        id="preferred-agent"
        value={preferred}
        disabled={busy}
        onChange={(e) => {
          const v = e.target.value;
          setPreferred(v);
          void perform(() => agents.savePreferred(v));
        }}
      >
        <option value="codex">Codex CLI</option>
        <option value="claude">Claude Code</option>
      </select>
      {!providers.length && !busy && (
        <Button type="button" onClick={() => void perform(async () => {})}>
          Retry settings
        </Button>
      )}
      <Concurrency />
      {providers.map(({ settings: s, version, message }) => (
        <fieldset key={s.provider} disabled={busy} className="agent-settings">
          <legend>
            {s.provider === 'codex' ? 'Codex CLI' : 'Claude Code'}
          </legend>
          <p>
            {version ?? 'Unavailable'} · {message}
          </p>
          <p className="git-target">
            {s.executable ?? 'No executable detected'}
          </p>
          <p className="muted">
            Choose a trusted installation. Selecting a file authorizes
            version/help commands. Forma checks its identity again before
            execution. Sign in using the provider CLI outside Forma.
          </p>
          <Button
            type="button"
            variant="secondary"
            onClick={() => void perform(() => agents.authorize(s.provider))}
          >
            Choose & authorize {s.provider} executable…
          </Button>
          <label htmlFor={`${s.provider}-runtime`}>
            Maximum runtime (seconds)
          </label>
          <input
            id={`${s.provider}-runtime`}
            type="number"
            min={1}
            max={7200}
            value={s.maxRuntime}
            onChange={(e) =>
              edit(s.provider, { maxRuntime: Number(e.target.value) })
            }
          />
          <label htmlFor={`${s.provider}-limit`}>
            Output/log limit (bytes)
          </label>
          <input
            id={`${s.provider}-limit`}
            type="number"
            min={1024}
            max={1048576}
            value={s.logLimit}
            onChange={(e) =>
              edit(s.provider, { logLimit: Number(e.target.value) })
            }
          />
          <label htmlFor={`${s.provider}-policy`}>Permission policy</label>
          <select
            id={`${s.provider}-policy`}
            value={s.policy}
            onChange={(e) =>
              edit(s.provider, {
                policy: e.target.value as AgentSettings['policy'],
              })
            }
          >
            <option value="restricted">
              Restricted — Codex workspace-write, no escalation
            </option>
            <option value="claude-tools">
              Claude restricted tools — no OS sandbox, no shell tools
            </option>
          </select>
          <label>
            <input
              type="checkbox"
              checked={s.requireOsIsolation ?? false}
              onChange={(e) =>
                edit(s.provider, { requireOsIsolation: e.target.checked })
              }
            />
            Require whole-provider OS isolation (unavailable; blocks execution)
          </label>
          <Button
            type="button"
            onClick={() => void perform(() => agents.saveSettings(s))}
          >
            Save {s.provider} defaults
          </Button>
        </fieldset>
      ))}
    </section>
  );
}
export function IssueAgent({
  issue,
  disabled,
}: {
  issue: Issue;
  disabled: boolean;
}) {
  const [expanded, setExpanded] = useState(false);
  return (
    <details
      className="issue-related"
      open={expanded}
      onToggle={(e) => setExpanded(e.currentTarget.open)}
    >
      <summary>AI Agent</summary>
      {expanded && (
        <Assignment key={issue.id} issue={issue} disabled={disabled} />
      )}
    </details>
  );
}
function Assignment({ issue, disabled }: { issue: Issue; disabled: boolean }) {
  const [provider, setProvider] = useState('codex');
  const [providers, setProviders] = useState<ProviderStatus[]>([]);
  const [preview, setPreview] = useState<AgentPreview | null>(null);
  const [elevated, setElevated] = useState(false);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState('');
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    let stale = false;
    Promise.all([agents.providers(), agents.preferred()])
      .then(([p, v]) => {
        if (!stale) {
          setProviders(p);
          setProvider(v);
        }
      })
      .catch((e) => {
        if (!stale) setError(errorMessage(e));
      })
      .finally(() => {
        if (!stale) setBusy(false);
      });
    return () => {
      stale = true;
    };
  }, []);
  const perform = async (action: () => Promise<void>) => {
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
    <section aria-label="Issue AI Agent">
      <p>
        Runs use the saved issue and a dedicated branch/worktree. Save issue
        edits first. Configure executables in AI agents & runs → Agent settings.
      </p>
      {error && (
        <p role="alert" className="form-error">
          {error}
        </p>
      )}
      {busy && <p role="status">Preparing agent…</p>}
      <label htmlFor="issue-agent">Coding agent</label>
      <select
        id="issue-agent"
        value={provider}
        disabled={busy || disabled}
        onChange={(e) => {
          setProvider(e.target.value);
          setPreview(null);
        }}
      >
        <option value="codex">Codex CLI</option>
        <option value="claude">Claude Code</option>
      </select>
      <p className="muted">
        {providers.find((p) => p.settings.provider === provider)?.message}
      </p>
      <Button
        id="agent-preview-task"
        type="button"
        variant="secondary"
        disabled={busy || disabled}
        onClick={() =>
          void perform(async () => {
            setElevated(false);
            setPreview(
              await agents.preview(issue.projectId, issue.id, provider),
            );
          })
        }
      >
        Preview agent task
      </Button>
      {preview && (
        <Dialog
          title={`Approve ${provider} execution?`}
          description="Review the exact prompt, executable, repository and permissions. This approval expires after five minutes and authorizes one run."
          onClose={() => setPreview(null)}
          onReturnFocus={() =>
            document.getElementById('agent-preview-task')?.focus()
          }
          busy={busy}
          className="issue-dialog"
        >
          <p className="git-target">
            {preview.executable} · {preview.version}
          </p>
          <p className="git-target">
            Repository: {preview.repository}
            <br />
            Base: {preview.baseCommit}
          </p>
          <p>
            {preview.policy} · {preview.maxRuntime}s maximum ·{' '}
            {preview.logLimit} log bytes
          </p>
          <p role="note" className="inline-confirm">
            {preview.warning}
          </p>
          <label htmlFor="agent-prompt">Task prompt (saved issue)</label>
          <textarea
            id="agent-prompt"
            readOnly
            value={preview.prompt}
            rows={12}
          />
          {provider === 'claude' && (
            <label>
              <input
                type="checkbox"
                checked={elevated}
                onChange={(e) => setElevated(e.target.checked)}
              />
              I separately approve running Claude without OS filesystem
              isolation.
            </label>
          )}
          {error && <p role="alert">{error}</p>}
          <div className="dialog-actions">
            <Button
              type="button"
              variant="secondary"
              disabled={busy}
              onClick={() => setPreview(null)}
            >
              Cancel approval
            </Button>
            <Button
              type="button"
              disabled={
                busy || disabled || (provider === 'claude' && !elevated)
              }
              onClick={() =>
                void perform(async () => {
                  await agents.enqueue(preview.token, elevated);
                  setPreview(null);
                  setRevision((v) => v + 1);
                })
              }
            >
              Approve & add to queue
            </Button>
            <Button
              type="button"
              disabled={
                busy || disabled || (provider === 'claude' && !elevated)
              }
              onClick={() =>
                void perform(async () => {
                  await agents.start(preview.token, elevated);
                  setPreview(null);
                  setRevision((v) => v + 1);
                })
              }
            >
              Approve & run once
            </Button>
          </div>
        </Dialog>
      )}
      <Runs issue={issue} revision={revision} />
    </section>
  );
}
function Progress({ log }: { log: string }) {
  const events = log
    .split('\n')
    .flatMap((line) => {
      try {
        const event: unknown = JSON.parse(line);
        if (
          event &&
          typeof event === 'object' &&
          'type' in event &&
          typeof event.type === 'string'
        )
          return [event.type.slice(0, 200)];
      } catch {
        /* Non-JSON stderr is available in bounded logs. */
      }
      return [];
    })
    .slice(-20);
  return events.length ? (
    <ol aria-label="Recent progress events">
      {events.map((type, i) => (
        <li key={i}>{type}</li>
      ))}
    </ol>
  ) : null;
}
function Runs({ issue, revision = 0 }: { issue?: Issue; revision?: number }) {
  const [runs, setRuns] = useState<AgentRun[]>([]);
  const [offset, setOffset] = useState(0);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [refresh, setRefresh] = useState(0);
  const [review, setReview] = useState<{
    run: AgentRun;
    result: AgentReview;
  } | null>(null);
  useEffect(() => {
    let stale = false;
    let timer: ReturnType<typeof setTimeout>;
    const load = async () => {
      try {
        const data = issue
          ? await agents.runs(issue.projectId, issue.id, offset)
          : await agents.recent(offset);
        if (!stale) {
          setRuns(data);
          setError('');
        }
      } catch (e) {
        if (!stale) setError(errorMessage(e));
      } finally {
        if (!stale) {
          setLoading(false);
          timer = setTimeout(() => void load(), 1000);
        }
      }
    };
    setLoading(true);
    void load();
    return () => {
      stale = true;
      clearTimeout(timer);
    };
  }, [issue, offset, revision, refresh]);
  const perform = async (action: () => Promise<void>) => {
    setBusy(true);
    setError('');
    try {
      await action();
      setRefresh((v) => v + 1);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <section aria-label="Agent runs" aria-busy={loading}>
      <h3>Agent runs {issue && `· ${issue.identifier}`}</h3>
      {loading && <p role="status">Loading runs…</p>}
      {error && (
        <p role="alert" className="form-error">
          {error}
        </p>
      )}
      {!loading && !runs.length && (
        <p className="muted">No agent runs on this page.</p>
      )}
      <Button
        type="button"
        variant="ghost"
        disabled={loading}
        onClick={() => setRefresh((v) => v + 1)}
      >
        Refresh runs
      </Button>
      {runs.map((run) => (
        <article className="activity-row agent-run" key={run.id}>
          <strong>
            {run.provider} · {run.status}
          </strong>
          <p className="muted">
            Issue {issue?.identifier ?? run.issueIdentifier} ·{' '}
            {Math.max(
              0,
              Math.floor(
                ((run.endedAt ? Date.parse(run.endedAt) : Date.now()) -
                  Date.parse(run.startedAt)) /
                  1000,
              ),
            )}
            s · Exit {run.exitCode ?? '—'} {run.review && `· ${run.review}`}
          </p>
          <p>{run.issueTitle}</p>
          <p>{run.result}</p>
          <Progress log={run.log} />
          <p className="git-target">
            {run.branch}
            <br />
            {run.worktree}
          </p>
          <details>
            <summary>Progress & bounded logs</summary>
            <pre className="agent-output">
              {run.log || 'No output received.'}
            </pre>
          </details>
          {isActive(run.status) ? (
            <Button
              type="button"
              variant="destructive"
              disabled={busy}
              onClick={() => void perform(() => agents.cancel(run.id))}
            >
              Cancel execution
            </Button>
          ) : (
            <Button
              type="button"
              variant="secondary"
              disabled={busy || !run.trusted}
              id={`agent-inspect-${run.id}`}
              onClick={() =>
                void perform(async () =>
                  setReview({ run, result: await agents.review(run.id) }),
                )
              }
            >
              Inspect result
            </Button>
          )}
          {!run.trusted && (
            <p className="muted">
              Restored history: worktree path access is disabled. Inspect
              retained files manually.
            </p>
          )}
        </article>
      ))}
      <div className="section-heading">
        <Button
          type="button"
          variant="secondary"
          disabled={loading || offset === 0}
          onClick={() => setOffset((v) => Math.max(0, v - 20))}
        >
          Previous runs
        </Button>
        <span>Page {offset / 20 + 1}</span>
        <Button
          type="button"
          variant="secondary"
          disabled={loading || runs.length < 20 || offset >= 10000}
          onClick={() => setOffset((v) => v + 20)}
        >
          Next runs
        </Button>
      </div>
      {review && (
        <Dialog
          title="Review agent result"
          description="Acceptance records your review for manual integration. The branch and worktree are retained after either decision."
          onClose={() => setReview(null)}
          onReturnFocus={() =>
            document.getElementById(`agent-inspect-${review.run.id}`)?.focus()
          }
          busy={busy}
          className="issue-dialog"
        >
          <p>
            {review.run.status} · Exit {review.run.exitCode ?? '—'}
          </p>
          <p>{review.run.result}</p>
          <p>
            +{review.result.additions} / −{review.result.deletions} lines{' '}
            {review.result.truncated && '· Display truncated'}
          </p>
          <ul>
            {review.result.files.map((file, i) => (
              <li className="git-target" key={i}>
                {file}
              </li>
            ))}
          </ul>
          <pre className="agent-output">
            {review.result.diff ||
              'No text diff. Check binary or ignored files manually.'}
          </pre>
          <p>
            Tests in agent logs are provider reports. Forma does not
            independently certify them. Inspect and integrate manually from{' '}
            <span className="git-target">{review.run.branch}</span>.
          </p>
          <IndependentTests runId={review.run.id} />
          {error && <p role="alert">{error}</p>}
          <div className="dialog-actions">
            <Button
              type="button"
              variant="secondary"
              disabled={busy}
              onClick={() => void perform(() => agents.open(review.run.id))}
            >
              Open worktree
            </Button>
            <Button
              type="button"
              variant="destructive"
              disabled={busy}
              onClick={() =>
                void perform(async () => {
                  await agents.decide(review.run.id, 'rejected');
                  setReview(null);
                })
              }
            >
              Reject result
            </Button>
            <Button
              type="button"
              disabled={busy}
              onClick={() =>
                void perform(async () => {
                  await agents.decide(review.run.id, 'accepted');
                  setReview(null);
                })
              }
            >
              Accept for manual integration
            </Button>
          </div>
        </Dialog>
      )}
    </section>
  );
}

function Concurrency() {
  const [maximum, setMaximum] = useState(1);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(true);
  useEffect(() => {
    let stale = false;
    agents
      .concurrency()
      .then((v) => {
        if (!stale) setMaximum(v);
      })
      .catch((e) => {
        if (!stale) setError(errorMessage(e));
      })
      .finally(() => {
        if (!stale) setBusy(false);
      });
    return () => {
      stale = true;
    };
  }, []);
  return (
    <fieldset disabled={busy}>
      <legend>Concurrent execution</legend>
      <label htmlFor="agent-concurrency">Maximum simultaneous runs</label>
      <select
        id="agent-concurrency"
        value={maximum}
        onChange={(e) => {
          const value = Number(e.target.value);
          setBusy(true);
          void agents
            .setConcurrency(value)
            .then(() => {
              setMaximum(value);
              setError('');
            })
            .catch((e) => setError(errorMessage(e)))
            .finally(() => setBusy(false));
        }}
      >
        <option value={1}>1 — conservative default</option>
        <option value={2}>2 — explicitly enable concurrent runs</option>
      </select>
      <p>
        Each process uses your OS account. Worktrees share Git metadata; CPU and
        RAM are not isolated.
      </p>
      {error && <p role="alert">{error}</p>}
    </fieldset>
  );
}
function Queue() {
  const [tasks, setTasks] = useState<QueuedTask[]>([]);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    let stale = false;
    let timer: ReturnType<typeof setTimeout>;
    const load = async () => {
      try {
        const values = await agents.queue();
        if (!stale) setTasks(values);
      } catch (e) {
        if (!stale) setError(errorMessage(e));
      } finally {
        if (!stale) {
          setLoading(false);
          timer = setTimeout(() => void load(), 1000);
        }
      }
    };
    void load();
    return () => {
      stale = true;
      clearTimeout(timer);
    };
  }, [revision]);
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
    <section aria-label="Approved task queue">
      <h3>Approved task queue</h3>
      <p>
        Approval expires in five minutes. Start explicitly; restart invalidates
        approvals. Start Approved Queue dispatches the currently approved tasks
        as slots become available. Later approvals require another Start action.
      </p>
      {loading && <p role="status">Loading approved tasks…</p>}
      {error && <p role="alert">{error}</p>}
      <Button
        disabled={busy}
        onClick={() => void perform(() => agents.startQueue(false))}
      >
        Start Next
      </Button>
      <Button
        disabled={busy}
        onClick={() => void perform(() => agents.startQueue(true))}
      >
        Start Approved Queue
      </Button>
      <Button
        disabled={busy}
        variant="secondary"
        onClick={() => void perform(() => agents.queueStop())}
      >
        Stop queue dispatch
      </Button>
      <Button
        disabled={busy}
        variant="ghost"
        onClick={() => setRevision((v) => v + 1)}
      >
        Refresh queue
      </Button>
      {!loading && !tasks.length && <p>No queued tasks.</p>}
      <ol>
        {tasks.map((task, index) => (
          <li key={task.id}>
            <strong>
              {task.title} · {task.provider} · {task.status}
            </strong>
            <p>
              {task.message} · Approval expires {task.expiresAt}
            </p>
            {task.status === 'Approved' && (
              <>
                <Button
                  disabled={busy || index === 0}
                  onClick={() =>
                    void perform(() =>
                      agents.queueMove(task.id, tasks[index - 1].id),
                    )
                  }
                >
                  Move earlier
                </Button>
                <Button
                  disabled={busy}
                  variant="destructive"
                  onClick={() =>
                    void perform(() => agents.queueCancel(task.id))
                  }
                >
                  Cancel queued task
                </Button>
              </>
            )}
          </li>
        ))}
      </ol>
    </section>
  );
}
