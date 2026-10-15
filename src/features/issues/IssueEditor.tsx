import { IssueAgent } from '../agents/Agents';
import { useEffect, useRef, useState } from 'react';
import { Dialog } from '../../components/ui/dialog';
import { Button } from '../../components/ui/button';
import { errorMessage } from '../../lib/ipc';
import type {
  ActivityPage,
  Issue,
  IssueInput,
  IssuePage,
  Project,
} from '../../lib/ipc';
import { useIssueStore, issueDetails, emptyPage } from './store';
import { defaultQuery, priorities, statuses } from './options';
import { MarkdownPreview } from './Markdown';
import { IssueGitActivity } from '../git/IssueGitActivity';

function Related({
  issue,
  onOpen,
  onCreate,
  disabled,
}: {
  issue: Issue;
  onOpen: (issue: Issue) => void;
  onCreate: () => void;
  disabled: boolean;
}) {
  const [children, setChildren] = useState<IssuePage>(emptyPage);
  const [activity, setActivity] = useState<ActivityPage>({
    events: [],
    total: 0,
  });
  const [childOffset, setChildOffset] = useState(0);
  const [activityOffset, setActivityOffset] = useState(0);
  const [revision, setRevision] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    let cancelled = false;
    Promise.all([
      issueDetails.children(issue.projectId, issue.id, childOffset),
      issueDetails.activity(issue.projectId, issue.id, activityOffset),
    ])
      .then(([c, a]) => {
        if (!cancelled) {
          setChildren(c);
          setActivity(a);
          setError(null);
        }
      })
      .catch((e) => {
        if (!cancelled) setError(errorMessage(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [issue.id, issue.projectId, childOffset, activityOffset, revision]);
  const page = (type: 'children' | 'activity', offset: number) => {
    setLoading(true);
    if (type === 'children') setChildOffset(offset);
    else setActivityOffset(offset);
  };
  return (
    <div className="issue-related" aria-busy={loading}>
      {!issue.parentId && (
        <section aria-label="Subtasks">
          <div className="section-heading">
            <h3>
              Subtasks <span className="muted">{children.total}</span>
            </h3>
            <Button
              type="button"
              variant="secondary"
              disabled={disabled || issue.archived}
              onClick={onCreate}
            >
              Add subtask
            </Button>
          </div>
          {!loading && !children.total && (
            <p className="muted">Break this issue into a few smaller steps.</p>
          )}
          {children.issues.map((child) => (
            <button
              type="button"
              className="subtask-row"
              key={child.id}
              disabled={disabled}
              onClick={() => onOpen(child)}
            >
              <span className="issue-identifier">{child.identifier}</span>{' '}
              {child.title}
              <span className="muted">
                {child.archived ? 'Archived' : statuses[child.status]}
              </span>
            </button>
          ))}
          {children.total > 20 && (
            <div className="pagination">
              <Button
                type="button"
                variant="ghost"
                disabled={loading || !childOffset}
                onClick={() => page('children', childOffset - 20)}
              >
                Previous subtasks
              </Button>
              <Button
                type="button"
                variant="ghost"
                disabled={loading || childOffset + 20 >= children.total}
                onClick={() => page('children', childOffset + 20)}
              >
                Next subtasks
              </Button>
            </div>
          )}
        </section>
      )}
      <section aria-label="Issue activity">
        <h3>Activity</h3>
        {activity.events.map((e) => (
          <div className="activity-row" key={e.id}>
            <span>{e.summary}</span>
            <time dateTime={e.createdAt}>
              {new Date(e.createdAt).toLocaleString()}
            </time>
          </div>
        ))}
        {activity.total > 20 && (
          <div className="pagination">
            <Button
              type="button"
              variant="ghost"
              disabled={loading || !activityOffset}
              onClick={() => page('activity', activityOffset - 20)}
            >
              Newer activity
            </Button>
            <Button
              type="button"
              variant="ghost"
              disabled={loading || activityOffset + 20 >= activity.total}
              onClick={() => page('activity', activityOffset + 20)}
            >
              Older activity
            </Button>
          </div>
        )}
      </section>
      {loading && <p role="status">Loading details…</p>}
      {error && (
        <div role="alert" className="form-error">
          {error}
          <Button
            type="button"
            variant="secondary"
            onClick={() => {
              setLoading(true);
              setRevision((v) => v + 1);
            }}
          >
            Retry details
          </Button>
        </div>
      )}
    </div>
  );
}
export function IssueEditor({
  project,
  issue,
  parent,
  onClose,
  onOpen,
  onCreate,
}: {
  project: Project;
  issue?: Issue;
  parent?: Issue;
  onClose: () => void;
  onOpen: (issue: Issue) => void;
  onCreate: (parent: Issue) => void;
}) {
  const initial: IssueInput = {
    projectId: project.id,
    title: issue?.title ?? '',
    description: issue?.description ?? '',
    status: issue?.status ?? 'todo',
    priority: issue?.priority ?? 'none',
    dueDate: issue?.dueDate ?? null,
    parentId: issue?.parentId ?? parent?.id ?? null,
    labelIds: issue?.labelIds ?? [],
    position: issue?.position ?? 0,
  };
  const [input, setInput] = useState<IssueInput>(initial);
  const original = useRef(JSON.stringify(initial));
  const dirty = JSON.stringify(input) !== original.current;
  const { labels, busy, mutationError, save, clearError } = useIssueStore();
  const [preview, setPreview] = useState(false);
  const [confirm, setConfirm] = useState<'delete' | 'discard' | null>(null);
  const [parentQuery, setParentQuery] = useState('');
  const [parents, setParents] = useState<Issue[]>(parent ? [parent] : []);
  const [parentOffset, setParentOffset] = useState(0);
  const [parentTotal, setParentTotal] = useState(0);
  const [parentError, setParentError] = useState<string | null>(null);
  const [parentRevision, setParentRevision] = useState(0);
  const [parentLoading, setParentLoading] = useState(true);
  const [currentParent, setCurrentParent] = useState<Issue | undefined>(parent);
  const form = useRef<HTMLFormElement>(null);
  useEffect(() => {
    let cancelled = false;
    issueDetails
      .parents({
        ...defaultQuery(project.id),
        search: parentQuery,
        offset: parentOffset,
        limit: 25,
      })
      .then((page) => {
        if (!cancelled) {
          setParents(
            page.issues.filter((p) => !p.parentId && p.id !== issue?.id),
          );
          setParentTotal(page.total);
          setParentError(null);
        }
      })
      .catch((e) => {
        if (!cancelled) setParentError(errorMessage(e));
      })
      .finally(() => {
        if (!cancelled) setParentLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [project.id, issue?.id, parentQuery, parentOffset, parentRevision]);
  useEffect(() => {
    if (!issue?.parentId) return;
    let cancelled = false;
    issueDetails
      .read(project.id, issue.parentId)
      .then((p) => {
        if (!cancelled) setCurrentParent(p);
      })
      .catch((e) => {
        if (!cancelled) setParentError(errorMessage(e));
      });
    return () => {
      cancelled = true;
    };
  }, [project.id, issue?.parentId, parentRevision]);
  const update = <K extends keyof IssueInput>(key: K, value: IssueInput[K]) =>
    setInput((old) => ({ ...old, [key]: value }));
  const close = () => {
    if (dirty) setConfirm('discard');
    else onClose();
  };
  const selectedParent =
    currentParent?.id === input.parentId
      ? currentParent
      : parents.find((p) => p.id === input.parentId);
  return (
    <Dialog
      className="issue-dialog"
      title={
        issue
          ? issue.identifier + ' · Edit issue'
          : parent
            ? 'New subtask'
            : 'New issue'
      }
      description={
        project.name + ' · Ctrl/⌘ + Enter to save. Markdown supported.'
      }
      onClose={close}
      busy={busy}
    >
      <form
        ref={form}
        onSubmit={async (e) => {
          e.preventDefault();
          if (await save(issue?.id ?? null, input)) onClose();
        }}
        onKeyDown={(e) => {
          if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            e.preventDefault();
            form.current?.requestSubmit();
          }
        }}
      >
        <fieldset disabled={busy} className="form-fields">
          <label htmlFor="issue-title">Title</label>
          <input
            id="issue-title"
            required
            maxLength={200}
            autoFocus
            placeholder="What needs to happen?"
            value={input.title}
            onChange={(e) => update('title', e.target.value)}
          />
          <div className="section-heading">
            <label htmlFor="issue-description">Description</label>
            <Button
              type="button"
              variant="ghost"
              aria-pressed={preview}
              onClick={() => setPreview((v) => !v)}
            >
              {preview ? 'Write Markdown' : 'Preview Markdown'}
            </Button>
          </div>
          {preview ? (
            <MarkdownPreview text={input.description} />
          ) : (
            <textarea
              id="issue-description"
              rows={6}
              maxLength={50000}
              value={input.description}
              onChange={(e) => update('description', e.target.value)}
              placeholder="Add context, a checklist, or a code snippet…"
            />
          )}
          <div className="form-columns">
            <div>
              <label htmlFor="issue-status">Status</label>
              <select
                id="issue-status"
                value={input.status}
                onChange={(e) =>
                  update('status', e.target.value as IssueInput['status'])
                }
              >
                {Object.entries(statuses).map(([v, n]) => (
                  <option key={v} value={v}>
                    {n}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label htmlFor="issue-priority">Priority</label>
              <select
                id="issue-priority"
                value={input.priority}
                onChange={(e) =>
                  update('priority', e.target.value as IssueInput['priority'])
                }
              >
                {Object.entries(priorities).map(([v, n]) => (
                  <option key={v} value={v}>
                    {n}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="form-columns">
            <div>
              <label htmlFor="issue-due">Due date</label>
              <input
                id="issue-due"
                type="date"
                value={input.dueDate ?? ''}
                onChange={(e) => update('dueDate', e.target.value || null)}
              />
            </div>
            <div>
              <label htmlFor="issue-order">Order</label>
              <input
                id="issue-order"
                type="number"
                min={0}
                max={1000000000}
                required
                value={input.position}
                onChange={(e) => update('position', Number(e.target.value))}
              />
            </div>
          </div>
          <label htmlFor="parent-search">Find parent issue</label>
          <input
            id="parent-search"
            type="search"
            maxLength={200}
            value={parentQuery}
            onChange={(e) => {
              setParentQuery(e.target.value);
              setParentOffset(0);
              setParentLoading(true);
            }}
            placeholder="Search top-level issues…"
          />
          <label htmlFor="issue-parent">Parent issue</label>
          <select
            id="issue-parent"
            value={input.parentId ?? ''}
            onChange={(e) => {
              setCurrentParent(parents.find((p) => p.id === e.target.value));
              update('parentId', e.target.value || null);
            }}
          >
            <option value="">No parent · top level</option>
            {input.parentId &&
              !parents.some((p) => p.id === input.parentId) && (
                <option value={input.parentId}>
                  {selectedParent
                    ? selectedParent.identifier + ' · ' + selectedParent.title
                    : 'Current parent'}
                </option>
              )}
            {parents.map((p) => (
              <option key={p.id} value={p.id}>
                {p.identifier} · {p.title}
              </option>
            ))}
          </select>
          <p className="muted small-text">
            One level of subtasks. Issues with subtasks must stay at the top
            level.
          </p>
          {parentLoading && <p role="status">Loading parent choices…</p>}
          {parentError && (
            <p role="alert">
              {parentError}
              <Button
                type="button"
                variant="ghost"
                onClick={() => {
                  setParentLoading(true);
                  setParentRevision((v) => v + 1);
                }}
              >
                Retry parents
              </Button>
            </p>
          )}
          {parentTotal > 25 && (
            <div className="pagination">
              <Button
                type="button"
                variant="ghost"
                disabled={parentLoading || !parentOffset}
                onClick={() => {
                  setParentOffset((v) => v - 25);
                  setParentLoading(true);
                }}
              >
                Previous parents
              </Button>
              <Button
                type="button"
                variant="ghost"
                disabled={parentLoading || parentOffset + 25 >= parentTotal}
                onClick={() => {
                  setParentOffset((v) => v + 25);
                  setParentLoading(true);
                }}
              >
                Next parents
              </Button>
            </div>
          )}
          <fieldset className="label-choices">
            <legend>Labels</legend>
            {labels.length ? (
              labels.map((l) => (
                <label className="label-choice" key={l.id}>
                  <input
                    type="checkbox"
                    checked={input.labelIds.includes(l.id)}
                    onChange={(e) =>
                      update(
                        'labelIds',
                        e.target.checked
                          ? [...input.labelIds, l.id]
                          : input.labelIds.filter((id) => id !== l.id),
                      )
                    }
                  />
                  <span className="label-dot" style={{ background: l.color }} />
                  {l.name}
                </label>
              ))
            ) : (
              <p className="muted">
                Create workspace labels from the project’s Labels button.
              </p>
            )}
          </fieldset>
        </fieldset>
        {mutationError && (
          <p role="alert" className="form-error">
            {mutationError}
          </p>
        )}
        <div className="dialog-actions">
          <Button
            type="button"
            variant="secondary"
            disabled={busy}
            onClick={close}
          >
            Cancel
          </Button>
          <Button disabled={busy || !input.title.trim()}>
            {busy ? 'Saving…' : issue ? 'Save changes' : 'Create issue'}
          </Button>
        </div>
      </form>
      {confirm && (
        <section
          className="inline-confirm"
          role="alertdialog"
          aria-label={
            confirm === 'delete' ? 'Confirm issue deletion' : 'Discard changes?'
          }
        >
          <p>
            {confirm === 'delete'
              ? 'Permanently delete this issue and its agent run history? Its subtasks will be kept as top-level issues. Agent worktrees are retained for manual inspection.'
              : 'Discard your unsaved changes?'}
          </p>
          <Button
            variant="secondary"
            autoFocus
            disabled={busy}
            onClick={() => setConfirm(null)}
          >
            Keep editing
          </Button>
          <Button
            variant="destructive"
            disabled={busy}
            onClick={async () => {
              if (confirm === 'discard') {
                onClose();
                return;
              }
              if (issue && (await issueDetails.delete(issue))) onClose();
            }}
          >
            {' '}
            {confirm === 'delete' ? 'Delete permanently' : 'Discard changes'}
          </Button>
        </section>
      )}
      {issue && (
        <>
          <div className="issue-meta">
            <span>Created {new Date(issue.createdAt).toLocaleString()}</span>
            <span>Updated {new Date(issue.updatedAt).toLocaleString()}</span>
          </div>
          <div className="issue-tools">
            <Button
              variant="secondary"
              disabled={busy || dirty}
              onClick={async () => {
                clearError();
                if (await issueDetails.archive(issue)) onClose();
              }}
            >
              {issue.archived ? 'Restore issue' : 'Archive issue'}
            </Button>
            <Button
              variant="destructive"
              disabled={busy}
              onClick={() => {
                clearError();
                setConfirm('delete');
              }}
            >
              Delete issue
            </Button>
          </div>
          {dirty && (
            <p className="muted small-text">
              Save or discard your changes before opening subtasks or archiving.
            </p>
          )}
          <IssueAgent issue={issue} disabled={busy || dirty} />
          <IssueGitActivity
            key={issue.id}
            issue={issue}
            disabled={busy || dirty}
          />
          <Related
            issue={issue}
            onOpen={onOpen}
            onCreate={() => onCreate(issue)}
            disabled={busy || dirty}
          />
        </>
      )}
    </Dialog>
  );
}
