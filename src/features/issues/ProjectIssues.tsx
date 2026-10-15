import { lazy, Suspense, useEffect, useRef, useState } from 'react';
import {
  ArrowLeft,
  Plus,
  Tag,
  Search,
  Circle,
  CheckCircle2,
  CornerDownRight,
} from 'lucide-react';
import { Button } from '../../components/ui/button';
import { ProjectIcon } from '../../components/ProjectIcon';
import { errorMessage } from '../../lib/ipc';
const Board = lazy(() =>
  import('./Board').then((module) => ({ default: module.Board })),
);
import type { Issue, IssueQuery, Project } from '../../lib/ipc';
import { useIssueStore, issuePreferences } from './store';
import { defaultQuery, priorities, statuses } from './options';
import { IssueEditor } from './IssueEditor';
import { LabelsDialog } from './LabelsDialog';
import { GitPanel } from '../git/GitPanel';

type Editor = { issue?: Issue; parent?: Issue; key: number };
export function ProjectIssues({
  project,
  onBack,
  onEdit,
}: {
  project: Project;
  onBack: () => void;
  onEdit: () => void;
}) {
  const [gitOpen, setGitOpen] = useState(false);
  const [view, setView] = useState<'list' | 'board'>('list');
  const viewGeneration = useRef(0);
  const [viewBusy, setViewBusy] = useState(false);
  const [viewError, setViewError] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    const generation = ++viewGeneration.current;
    void issuePreferences
      .read(project.id)
      .then((value) => {
        if (!cancelled && generation === viewGeneration.current) setView(value);
      })
      .catch((error) => {
        if (!cancelled && generation === viewGeneration.current)
          setViewError(errorMessage(error));
      });
    return () => {
      cancelled = true;
    };
  }, [project.id]);
  const selectView = async (value: 'list' | 'board') => {
    if (viewBusy) return;
    ++viewGeneration.current;
    setViewBusy(true);
    try {
      await issuePreferences.save(project.id, value);
      setView(value);
      setViewError(null);
    } catch (error) {
      setViewError(errorMessage(error));
    } finally {
      setViewBusy(false);
    }
  };
  const [query, setQuery] = useState(() => defaultQuery(project.id));
  const [editor, setEditor] = useState<Editor | null>(null);
  const [manageLabels, setManageLabels] = useState(false);
  const { page, labels, loading, error, busy, load, refresh, clearError } =
    useIssueStore();
  const search = useRef<HTMLInputElement>(null);
  const key = useRef(0);
  const open = (issue?: Issue, parent?: Issue) => {
    clearError();
    setEditor({ issue, parent, key: ++key.current });
  };
  useEffect(() => {
    void load(query, project.workspaceId, view === 'board');
  }, [query, project.workspaceId, load, view]);
  useEffect(() => {
    const shortcut = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (
        editor ||
        manageLabels ||
        busy ||
        loading ||
        !!error ||
        target.closest(
          'input,textarea,select,[contenteditable="true"],[role="dialog"]',
        ) ||
        e.ctrlKey ||
        e.metaKey ||
        e.altKey
      )
        return;
      if (e.key === 'n') {
        e.preventDefault();
        clearError();
        setEditor({ key: ++key.current });
      }
      if (e.key === '/') {
        e.preventDefault();
        search.current?.focus();
      }
    };
    window.addEventListener('keydown', shortcut);
    return () => window.removeEventListener('keydown', shortcut);
  }, [editor, manageLabels, busy, loading, error, clearError]);
  const filter = <K extends keyof IssueQuery>(key: K, value: IssueQuery[K]) =>
    setQuery((q) => ({ ...q, [key]: value, offset: 0 }));
  return (
    <div className="project-detail">
      <div className="detail-navigation">
        <Button variant="ghost" onClick={onBack}>
          <ArrowLeft size={15} />
          All projects
        </Button>
        <Button variant="ghost" onClick={onEdit}>
          Project settings
        </Button>
      </div>
      <div className="page-heading">
        <div>
          <div className="eyebrow accent">PROJECT / ISSUES</div>
          <h1>
            <ProjectIcon icon={project.icon} color={project.color} />
            {project.name}
          </h1>
          <p className="muted">
            {project.description || 'Turn ideas into small, steady steps.'}
          </p>
          {project.archived && (
            <p className="muted">
              Archived project · Your issues remain available.
            </p>
          )}
        </div>
        <Button disabled={busy || loading || !!error} onClick={() => open()}>
          <Plus size={16} />
          New issue
        </Button>
      </div>
      <details
        open={gitOpen}
        onToggle={(e) => setGitOpen(e.currentTarget.open)}
        className="project-git"
      >
        <summary>Git repository</summary>
        {gitOpen && <GitPanel key={project.id} projectId={project.id} />}
      </details>
      <div className="issue-summary">
        <span>
          <strong>{page.projectTotal}</strong>{' '}
          {query.archived ? 'archived' : 'active'} issues
        </span>
        <span>
          <CheckCircle2 size={15} />
          <strong>{page.done}</strong> done
        </span>
        <span className="muted">N to create · / to search</span>
        <Button
          variant="ghost"
          disabled={busy || loading || !!error}
          onClick={() => {
            clearError();
            setManageLabels(true);
          }}
        >
          <Tag size={15} />
          Labels
        </Button>
      </div>
      <div className="view-switch" role="group" aria-label="Project view">
        <Button
          variant={view === 'list' ? 'default' : 'secondary'}
          aria-pressed={view === 'list'}
          disabled={busy || viewBusy}
          onClick={() => void selectView('list')}
        >
          List
        </Button>
        <Button
          variant={view === 'board' ? 'default' : 'secondary'}
          aria-pressed={view === 'board'}
          disabled={busy || viewBusy}
          onClick={() => void selectView('board')}
        >
          Board
        </Button>
      </div>
      {viewError && (
        <p role="alert" className="error-banner">
          {viewError}
        </p>
      )}
      <div className="issue-filters">
        <div className="search-field">
          <Search size={16} />
          <input
            ref={search}
            aria-label="Search issues"
            placeholder="Search issues or identifiers…"
            maxLength={200}
            value={query.search}
            onChange={(e) => filter('search', e.target.value)}
          />
        </div>
        <select
          aria-label="Filter issue status"
          value={query.status ?? ''}
          onChange={(e) =>
            filter('status', (e.target.value || null) as IssueQuery['status'])
          }
        >
          <option value="">All statuses</option>
          {Object.entries(statuses).map(([v, n]) => (
            <option key={v} value={v}>
              {n}
            </option>
          ))}
        </select>
        <select
          aria-label="Filter issue priority"
          value={query.priority ?? ''}
          onChange={(e) =>
            filter(
              'priority',
              (e.target.value || null) as IssueQuery['priority'],
            )
          }
        >
          <option value="">All priorities</option>
          {Object.entries(priorities).map(([v, n]) => (
            <option key={v} value={v}>
              {n}
            </option>
          ))}
        </select>
        <select
          disabled={view === 'board'}
          aria-label="Sort issues"
          value={query.sort}
          onChange={(e) => filter('sort', e.target.value as IssueQuery['sort'])}
        >
          <option value="position">Manual order</option>
          <option value="priority">Priority</option>
          <option value="newest">Newest first</option>
          <option value="updated">Recently updated</option>
          <option value="due">Due date</option>
        </select>
        <select
          aria-label="Filter issue label"
          value={query.labelId ?? ''}
          onChange={(e) => filter('labelId', e.target.value || null)}
        >
          <option value="">All labels</option>
          {labels.map((label) => (
            <option key={label.id} value={label.id}>
              {label.name}
            </option>
          ))}
        </select>
        <label>
          Due from
          <input
            type="date"
            aria-label="Due from"
            disabled={query.undated}
            value={query.dueFrom ?? ''}
            onChange={(e) => filter('dueFrom', e.target.value || null)}
          />
        </label>
        <label>
          Due through
          <input
            type="date"
            aria-label="Due through"
            disabled={query.undated}
            value={query.dueTo ?? ''}
            onChange={(e) => filter('dueTo', e.target.value || null)}
          />
        </label>
        <label className="archive-toggle">
          <input
            type="checkbox"
            checked={query.undated ?? false}
            onChange={(e) =>
              setQuery((q) => ({
                ...q,
                undated: e.target.checked,
                dueFrom: null,
                dueTo: null,
                offset: 0,
              }))
            }
          />
          No due date
        </label>
        <Button
          variant="ghost"
          onClick={() => setQuery(defaultQuery(project.id))}
        >
          Clear all filters
        </Button>
        <label className="archive-toggle">
          <input
            type="checkbox"
            checked={query.archived}
            onChange={(e) => filter('archived', e.target.checked)}
          />
          Archived
        </label>
      </div>
      {error && (
        <div role="alert" className="error-banner">
          <p>{error}</p>
          <Button variant="secondary" onClick={() => void refresh()}>
            Retry issues
          </Button>
        </div>
      )}
      {view === 'board' && loading && <p role="status">Loading board…</p>}
      {view === 'board' ? (
        !error && (
          <Suspense fallback={<p role="status">Loading board…</p>}>
            <Board onOpen={(issue) => open(issue)} />
          </Suspense>
        )
      ) : (
        <section
          className="issues-panel"
          aria-label="Project issues"
          aria-busy={loading}
        >
          {loading ? (
            <div className="issue-placeholder" role="status">
              Loading issues…
            </div>
          ) : !error && page.issues.length === 0 ? (
            <div className="empty-state">
              <Circle size={30} />
              <h2>
                {query.search ||
                query.status ||
                query.priority ||
                query.labelId ||
                query.dueFrom ||
                query.dueTo ||
                query.undated
                  ? 'No matching issues'
                  : query.archived
                    ? 'Nothing archived'
                    : 'A clear path starts here.'}
              </h2>
              <p>
                {query.search ||
                query.status ||
                query.priority ||
                query.labelId ||
                query.dueFrom ||
                query.dueTo ||
                query.undated
                  ? 'Try another search or clear the filters.'
                  : query.archived
                    ? 'Archived issues can be restored anytime.'
                    : 'Add an issue, then break it into a few manageable steps.'}
              </p>
              {query.search ||
              query.status ||
              query.priority ||
              query.labelId ||
              query.dueFrom ||
              query.dueTo ||
              query.undated ? (
                <Button
                  variant="secondary"
                  onClick={() => setQuery(defaultQuery(project.id))}
                >
                  Clear filters
                </Button>
              ) : (
                !query.archived && (
                  <Button onClick={() => open()}>
                    Create your first issue
                  </Button>
                )
              )}
            </div>
          ) : (
            !error && (
              <>
                <div className="issue-list-heading">
                  <span>Issue</span>
                  <span>Status / Priority / Due</span>
                </div>
                <ul className="issue-list">
                  {page.issues.map((issue) => (
                    <li key={issue.id}>
                      <button
                        className="issue-row"
                        disabled={busy}
                        onClick={() => open(issue)}
                      >
                        <span
                          className={'issue-status-dot status-' + issue.status}
                        >
                          {issue.status === 'done' ? (
                            <CheckCircle2 size={17} />
                          ) : (
                            <Circle size={17} />
                          )}
                        </span>
                        <span className="issue-identifier">
                          {issue.identifier}
                        </span>
                        <span className="issue-row-main">
                          <strong>
                            {issue.parentId && <CornerDownRight size={13} />}{' '}
                            {issue.title}
                          </strong>
                          <span className="issue-row-labels">
                            {issue.labelIds.map((id) => {
                              const l = labels.find((l) => l.id === id);
                              return l ? (
                                <span className="issue-label" key={id}>
                                  <span
                                    className="label-dot"
                                    style={{ background: l.color }}
                                  />
                                  {l.name}
                                </span>
                              ) : null;
                            })}
                          </span>
                        </span>
                        <span className="issue-row-status">
                          {statuses[issue.status]}
                        </span>
                        <span className={'priority priority-' + issue.priority}>
                          {priorities[issue.priority]}
                        </span>
                        <time
                          className="issue-due"
                          dateTime={issue.dueDate ?? undefined}
                        >
                          {issue.dueDate ?? '—'}
                        </time>
                      </button>
                    </li>
                  ))}
                </ul>
              </>
            )
          )}
        </section>
      )}
      <div className="pagination">
        <span className="muted">
          {page.total} matching issues
          {page.issues.length > 0 &&
            ` · ${query.offset + 1}–${Math.min(query.offset + query.limit, page.total)}`}
        </span>
        <Button
          variant="secondary"
          disabled={loading || query.offset === 0}
          onClick={() =>
            setQuery((q) => ({ ...q, offset: Math.max(0, q.offset - q.limit) }))
          }
        >
          Previous
        </Button>
        <Button
          variant="secondary"
          disabled={loading || query.offset + query.limit >= page.total}
          onClick={() =>
            setQuery((q) => ({ ...q, offset: q.offset + q.limit }))
          }
        >
          Next
        </Button>
        {!loading && query.offset > 0 && page.issues.length === 0 && (
          <Button variant="ghost" onClick={() => filter('offset', 0)}>
            Return to first page
          </Button>
        )}
      </div>
      {editor && (
        <IssueEditor
          key={editor.key}
          project={project}
          issue={editor.issue}
          parent={editor.parent}
          onClose={() => {
            clearError();
            setEditor(null);
          }}
          onOpen={(issue) => open(issue)}
          onCreate={(parent) => open(undefined, parent)}
        />
      )}
      {manageLabels && (
        <LabelsDialog
          workspaceId={project.workspaceId}
          onClose={() => {
            clearError();
            setManageLabels(false);
          }}
        />
      )}
    </div>
  );
}
