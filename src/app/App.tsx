import { version } from '../../package.json';
import { AgentHub } from '../features/agents/Agents';
import { useEffect, useMemo, useState } from 'react';
import {
  Archive,
  ArrowUpRight,
  CheckCircle2,
  ChevronRight,
  Circle,
  FolderKanban,
  Layers,
  LoaderCircle,
  Monitor,
  Moon,
  Pencil,
  Plus,
  Search,
  ShieldCheck,
  Sun,
  Trash2,
  X,
} from 'lucide-react';
import { ProjectIssues } from '../features/issues/ProjectIssues';
import { useAppStore } from './store';
import { applyTheme, readTheme } from './theme';
import type { Theme } from './theme';
import { api } from '../lib/ipc';
import type { Project, Workspace } from '../lib/ipc';
import { Button } from '../components/ui/button';
import { ProjectIcon } from '../components/ProjectIcon';
import { ProjectDialog } from '../features/projects/ProjectDialog';
import { WorkspaceDialog } from '../features/workspaces/WorkspaceDialog';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { BackupControls } from '../features/backup/BackupControls';
type Modal =
  | { kind: 'workspace'; workspace?: Workspace }
  | { kind: 'project'; project?: Project }
  | { kind: 'deleteWorkspace'; workspace: Workspace }
  | { kind: 'deleteProject'; project: Project }
  | null;
export default function App() {
  const {
    workspaces,
    activeWorkspaceId,
    projects,
    loading,
    busy,
    ready,
    error,
    refresh,
    mutate,
    clearError,
  } = useAppStore();
  const [modal, setModal] = useState<Modal>(null);
  const [page, setPage] = useState<'projects' | 'archive'>('projects');
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(
    null,
  );
  const selectedProject = projects.find((p) => p.id === selectedProjectId);
  const [query, setQuery] = useState('');
  const [status, setStatus] = useState('all');
  const [theme, setTheme] = useState<Theme>(readTheme);
  const savedTheme = useAppStore((s) => s.theme);
  useEffect(() => {
    if (savedTheme) setTheme(savedTheme);
    else if (ready && savedTheme === null)
      void mutate(() => api.setTheme(readTheme()));
  }, [savedTheme, ready, mutate]);
  const workspace = workspaces.find((item) => item.id === activeWorkspaceId);
  useEffect(() => {
    void refresh();
  }, [refresh]);
  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const update = () => applyTheme(theme, media.matches);
    update();
    media.addEventListener('change', update);
    return () => media.removeEventListener('change', update);
  }, [theme]);
  const visible = useMemo(
    () =>
      projects.filter(
        (project) =>
          project.archived === (page === 'archive') &&
          (status === 'all' || project.status === status) &&
          (project.name + ' ' + project.description)
            .toLowerCase()
            .includes(query.toLowerCase()),
      ),
    [projects, page, status, query],
  );
  const activeProjects = projects.filter((project) => !project.archived);
  const open = (value: Modal) => {
    clearError();
    setModal(value);
  };
  const close = () => {
    clearError();
    setModal(null);
  };
  const switchPage = (value: typeof page) => {
    setSelectedProjectId(null);
    setPage(value);
    setQuery('');
    setStatus('all');
  };
  return (
    <div className="app-shell">
      <aside className="sidebar" aria-label="Workspace navigation">
        <div className="brand">
          <span className="brand-mark">
            <Layers size={21} />
          </span>
          forma<span className="version">{version}</span>
        </div>
        <div className="workspace-switcher">
          <label htmlFor="workspace-select" className="eyebrow">
            WORKSPACE
          </label>
          <select
            id="workspace-select"
            disabled={busy || loading || !workspaces.length}
            value={activeWorkspaceId ?? ''}
            onChange={(event) => {
              setSelectedProjectId(null);
              void mutate(() => api.selectWorkspace(event.target.value));
            }}
            aria-label="Active workspace"
          >
            {!workspaces.length && <option value="">No workspace yet</option>}
            {workspaces.map((item) => (
              <option value={item.id} key={item.id}>
                {item.name}
              </option>
            ))}
          </select>
        </div>
        <nav>
          <button
            className={page === 'projects' ? 'nav-item selected' : 'nav-item'}
            onClick={() => switchPage('projects')}
          >
            <FolderKanban size={17} />
            Projects<span className="count">{activeProjects.length}</span>
          </button>
          <button
            className={page === 'archive' ? 'nav-item selected' : 'nav-item'}
            onClick={() => switchPage('archive')}
          >
            <Archive size={17} />
            Archive
            <span className="count">
              {projects.filter((p) => p.archived).length}
            </span>
          </button>
        </nav>
        <div className="sidebar-section">
          <div className="section-heading">
            <span className="eyebrow">YOUR PROJECTS</span>
            <button
              className="icon-button"
              aria-label="New project"
              disabled={!workspace || busy}
              onClick={() => open({ kind: 'project' })}
            >
              <Plus size={15} />
            </button>
          </div>
          <div className="project-nav">
            {activeProjects.map((project) => (
              <button
                key={project.id}
                className="project-nav-item"
                disabled={busy}
                onClick={() => setSelectedProjectId(project.id)}
              >
                <ProjectIcon icon={project.icon} color={project.color} small />
                <span>{project.name}</span>
              </button>
            ))}
            {!activeProjects.length && (
              <p className="sidebar-empty">Your projects will appear here.</p>
            )}
          </div>
        </div>
        <div className="sidebar-bottom">
          <div className="workspace-actions">
            <Button
              variant="ghost"
              disabled={!ready || busy}
              onClick={() => open({ kind: 'workspace' })}
            >
              <Plus size={15} />
              New workspace
            </Button>
            {workspace && (
              <>
                <button
                  className="icon-button"
                  aria-label="Rename workspace"
                  disabled={busy}
                  onClick={() => open({ kind: 'workspace', workspace })}
                >
                  <Pencil size={14} />
                </button>
                <button
                  className="icon-button"
                  aria-label="Delete workspace"
                  disabled={busy}
                  onClick={() => open({ kind: 'deleteWorkspace', workspace })}
                >
                  <Trash2 size={14} />
                </button>
              </>
            )}
          </div>
          <AgentHub />
          <BackupControls
            onRestored={() => {
              setSelectedProjectId(null);
              setPage('projects');
              setQuery('');
              setStatus('all');
              setModal(null);
            }}
          />
          <div className="theme-row">
            <span className="local-label">
              <ShieldCheck size={14} />
              Local-first
            </span>
            <div role="group" aria-label="Appearance" className="theme-switch">
              {(
                [
                  { value: 'light', Icon: Sun },
                  { value: 'dark', Icon: Moon },
                  { value: 'system', Icon: Monitor },
                ] as const
              ).map(({ value, Icon }) => (
                <button
                  key={value}
                  className="icon-button"
                  aria-label={value + ' theme'}
                  aria-pressed={theme === value}
                  disabled={busy}
                  onClick={() => void mutate(() => api.setTheme(value))}
                >
                  <Icon size={14} />
                </button>
              ))}
            </div>
          </div>
        </div>
      </aside>
      <div className="main-shell">
        <header className="topbar">
          <div className="breadcrumbs">
            <span>{workspace?.name ?? 'Workspace'}</span>
            <ChevronRight size={14} />
            <strong>
              {selectedProject?.name ??
                (page === 'projects' ? 'Projects' : 'Archive')}
            </strong>
          </div>
          <span className="device-badge">
            <span /> On this device
          </span>
        </header>
        <main id="main-content">
          {selectedProject ? (
            <ProjectIssues
              key={selectedProject.id}
              project={selectedProject}
              onBack={() => setSelectedProjectId(null)}
              onEdit={() => open({ kind: 'project', project: selectedProject })}
            />
          ) : (
            <>
              <div className="page-heading">
                <div>
                  <div className="eyebrow accent">
                    A LITTLE STRUCTURE. A LOT OF POSSIBILITY.
                  </div>
                  <h1>
                    {page === 'projects' ? 'Projects' : 'Archive'}
                    <span className="heading-count">
                      {page === 'projects'
                        ? activeProjects.length
                        : projects.filter((p) => p.archived).length}
                    </span>
                  </h1>
                  <p className="muted">
                    {page === 'projects'
                      ? 'A clear space for everything you’re building.'
                      : 'Past projects, ready whenever you need them.'}
                  </p>
                </div>
                {workspace && page === 'projects' && (
                  <Button
                    disabled={busy || loading}
                    onClick={() => open({ kind: 'project' })}
                  >
                    <Plus size={16} />
                    New project
                  </Button>
                )}
              </div>
              {error && !modal && (
                <div role="alert" className="error-banner">
                  <div>
                    <strong>Unable to complete request</strong>
                    <p>{error}</p>
                  </div>
                  <Button
                    variant="secondary"
                    disabled={loading || busy}
                    onClick={() => void refresh()}
                  >
                    Retry
                  </Button>
                </div>
              )}
              {loading && (
                <div role="status" className="loading-state">
                  <LoaderCircle className="spin" size={24} />
                  Loading your workspace…
                </div>
              )}
              {!loading && ready && !workspace && (
                <section className="empty-state welcome">
                  <div className="empty-art">
                    <Layers size={38} />
                  </div>
                  <span className="eyebrow accent">WELCOME TO FORMA</span>
                  <h2>Great work starts with a little space.</h2>
                  <p>
                    Create your first workspace to organize projects.
                    <br />
                    Your data lives here, on your device.
                  </p>
                  <p className="muted">
                    Backups are unencrypted. AI agents are optional and need
                    approval; their worktrees are not security sandboxes.
                  </p>
                  <Button onClick={() => open({ kind: 'workspace' })}>
                    <Plus size={16} />
                    Create workspace
                  </Button>
                  <div className="empty-footnote">
                    <ShieldCheck size={14} />
                    No account. No cloud. Just your work.
                  </div>
                </section>
              )}
              {ready && workspace && !loading && (
                <>
                  <div className="toolbar">
                    <div className="view-label">
                      <FolderKanban size={16} />
                      {page === 'projects'
                        ? 'All projects'
                        : 'Archived projects'}
                    </div>
                    <div className="filters">
                      <div className="search-field">
                        <Search size={15} />
                        <input
                          aria-label="Search projects"
                          value={query}
                          onChange={(e) => setQuery(e.target.value)}
                          placeholder="Search projects…"
                        />
                        {query && (
                          <button
                            className="icon-button"
                            aria-label="Clear search"
                            onClick={() => setQuery('')}
                          >
                            <X size={13} />
                          </button>
                        )}
                      </div>
                      <select
                        aria-label="Filter by status"
                        value={status}
                        onChange={(e) => setStatus(e.target.value)}
                      >
                        <option value="all">All statuses</option>
                        <option value="planned">Planned</option>
                        <option value="active">Active</option>
                        <option value="paused">Paused</option>
                        <option value="completed">Completed</option>
                      </select>
                    </div>
                  </div>
                  {visible.length ? (
                    <div className="project-grid">
                      {visible.map((project) => (
                        <article className="project-card" key={project.id}>
                          <div className="card-top">
                            <ProjectIcon
                              icon={project.icon}
                              color={project.color}
                            />
                            <span className={'status status-' + project.status}>
                              {project.status === 'completed' ? (
                                <CheckCircle2 size={12} />
                              ) : (
                                <Circle size={10} />
                              )}{' '}
                              {project.status}
                            </span>
                          </div>
                          <button
                            className="project-title"
                            disabled={busy}
                            onClick={() => setSelectedProjectId(project.id)}
                          >
                            {project.name}
                            <ArrowUpRight size={16} />
                          </button>
                          <p className="project-description">
                            {project.description ||
                              'No description yet. Give this project a little context.'}
                          </p>
                          <div className="card-footer">
                            <span>
                              Updated{' '}
                              {new Date(project.updatedAt).toLocaleDateString(
                                undefined,
                                { month: 'short', day: 'numeric' },
                              )}
                            </span>
                            <div className="card-actions">
                              <button
                                className="icon-button"
                                aria-label={'Edit ' + project.name}
                                disabled={busy}
                                onClick={() =>
                                  open({ kind: 'project', project })
                                }
                              >
                                <Pencil size={14} />
                              </button>
                              <button
                                className="icon-button"
                                aria-label={
                                  (project.archived ? 'Restore ' : 'Archive ') +
                                  project.name
                                }
                                disabled={busy}
                                onClick={() =>
                                  void mutate(() =>
                                    api.archiveProject(
                                      project.id,
                                      !project.archived,
                                    ),
                                  )
                                }
                              >
                                <Archive size={14} />
                              </button>
                              <button
                                className="icon-button danger-hover"
                                aria-label={'Delete ' + project.name}
                                disabled={busy}
                                onClick={() =>
                                  open({ kind: 'deleteProject', project })
                                }
                              >
                                <Trash2 size={14} />
                              </button>
                            </div>
                          </div>
                        </article>
                      ))}
                    </div>
                  ) : (
                    <section className="empty-state">
                      <div className="empty-art">
                        {page === 'archive' ? (
                          <Archive size={32} />
                        ) : (
                          <FolderKanban size={32} />
                        )}
                      </div>
                      <h2>
                        {query || status !== 'all'
                          ? 'No matching projects'
                          : page === 'archive'
                            ? 'A fresh start, with nothing archived.'
                            : 'Make room for your next idea.'}
                      </h2>
                      <p>
                        {query || status !== 'all'
                          ? 'Try a different search or status filter.'
                          : page === 'archive'
                            ? 'Projects you archive will be kept here.'
                            : 'Create a project and give your work a clear direction.'}
                      </p>
                      {query || status !== 'all' ? (
                        <Button
                          variant="secondary"
                          onClick={() => {
                            setQuery('');
                            setStatus('all');
                          }}
                        >
                          Clear filters
                        </Button>
                      ) : (
                        page === 'projects' && (
                          <Button onClick={() => open({ kind: 'project' })}>
                            <Plus size={16} />
                            Create your first project
                          </Button>
                        )
                      )}
                    </section>
                  )}
                  <footer className="content-footer">
                    <span>
                      {visible.length}{' '}
                      {visible.length === 1 ? 'project' : 'projects'}
                    </span>
                    <span>
                      <ShieldCheck size={13} />
                      Saved locally. Yours by design.
                    </span>
                  </footer>
                </>
              )}
            </>
          )}
        </main>
      </div>
      {modal?.kind === 'workspace' && (
        <WorkspaceDialog workspace={modal.workspace} onClose={close} />
      )}
      {modal?.kind === 'project' && workspace && (
        <ProjectDialog
          project={modal.project}
          workspaceId={workspace.id}
          onClose={close}
        />
      )}
      {modal?.kind === 'deleteWorkspace' && (
        <ConfirmDialog
          title={'Delete “' + modal.workspace.name + '”?'}
          description="This permanently deletes the workspace and all its projects, issues, labels, activity, and agent run history. Agent worktrees are retained for manual inspection. This cannot be undone."
          action={() => api.deleteWorkspace(modal.workspace.id)}
          onClose={close}
        />
      )}
      {modal?.kind === 'deleteProject' && (
        <ConfirmDialog
          title={'Delete “' + modal.project.name + '”?'}
          description="This permanently deletes the project, its issues, and agent run history. Agent worktrees are retained for manual inspection. This cannot be undone. You can archive the project instead to keep it."
          action={() => api.deleteProject(modal.project.id)}
          onClose={close}
        />
      )}
      {busy && (
        <div className="save-indicator" role="status">
          <LoaderCircle size={14} className="spin" />
          Saving…
        </div>
      )}
    </div>
  );
}
