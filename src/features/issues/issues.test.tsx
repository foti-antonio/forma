import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { api } from '../../lib/ipc';
import type { Issue, Project, Label } from '../../lib/ipc';
import { useIssueStore, emptyPage } from './store';
import { defaultQuery } from './options';
import { IssueEditor } from './IssueEditor';
import { ProjectIssues } from './ProjectIssues';
import { LabelsDialog } from './LabelsDialog';
import { MarkdownPreview } from './Markdown';
vi.mock('../../lib/ipc', async (original) => ({
  ...(await original<typeof import('../../lib/ipc')>()),
  api: {
    projectView: vi.fn(),
    setProjectView: vi.fn(),
    issues: vi.fn(),
    issue: vi.fn(),
    saveIssue: vi.fn(),
    labels: vi.fn(),
    saveLabel: vi.fn(),
    deleteLabel: vi.fn(),
    subtasks: vi.fn(),
    activity: vi.fn(),
    archiveIssue: vi.fn(),
    deleteIssue: vi.fn(),
  },
}));
const project: Project = {
  id: 'project',
  workspaceId: 'workspace',
  name: 'Engine',
  description: 'Ship it',
  icon: 'code',
  color: '#112233',
  status: 'active',
  archived: false,
  createdAt: '2026-10-15T00:00:00Z',
  updatedAt: '2026-10-15T00:00:00Z',
};
const issue: Issue = {
  id: 'issue',
  projectId: project.id,
  identifier: 'F1-1',
  number: 1,
  title: 'Ship engine',
  description: '# Plan',
  status: 'todo',
  priority: 'high',
  dueDate: null,
  parentId: null,
  labelIds: [],
  position: 0,
  archived: false,
  createdAt: project.createdAt,
  updatedAt: project.updatedAt,
};
const label: Label = {
  id: 'bug',
  workspaceId: project.workspaceId,
  name: 'Bug',
  color: '#ff0000',
  createdAt: project.createdAt,
  updatedAt: project.updatedAt,
};
beforeEach(() => {
  vi.resetAllMocks();
  useIssueStore.setState({
    query: null,
    workspaceId: project.workspaceId,
    page: emptyPage,
    labels: [label],
    busy: false,
    loading: false,
    error: null,
    mutationError: null,
    generation: 0,
  });
  vi.mocked(api.projectView).mockResolvedValue('list');
  vi.mocked(api.setProjectView).mockResolvedValue(undefined);
  vi.mocked(api.issues).mockResolvedValue(emptyPage);
  vi.mocked(api.labels).mockResolvedValue([label]);
  vi.mocked(api.subtasks).mockResolvedValue(emptyPage);
  vi.mocked(api.activity).mockResolvedValue({ events: [], total: 0 });
  vi.mocked(api.saveIssue).mockResolvedValue(issue);
});
afterEach(cleanup);
it('creates a complete issue with labels and keyboard save', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  render(
    <IssueEditor
      project={project}
      onClose={close}
      onOpen={vi.fn()}
      onCreate={vi.fn()}
    />,
  );
  await user.type(screen.getByLabelText('Title'), 'New task');
  await user.type(screen.getByLabelText('Description'), '**Why** this matters');
  await user.selectOptions(screen.getByLabelText('Status'), 'in_progress');
  await user.selectOptions(screen.getByLabelText('Priority'), 'urgent');
  await user.click(screen.getByLabelText('Bug'));
  await user.type(screen.getByLabelText('Order'), '5');
  await user.keyboard('{Control>}{Enter}{/Control}');
  expect(api.saveIssue).toHaveBeenCalledWith(
    null,
    expect.objectContaining({
      projectId: project.id,
      title: 'New task',
      description: '**Why** this matters',
      status: 'in_progress',
      priority: 'urgent',
      labelIds: ['bug'],
      position: 5,
      parentId: null,
      dueDate: null,
    }),
  );
  expect(close).toHaveBeenCalledOnce();
});
it('retains edits and label choices after errors and succeeds on retry', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  vi.mocked(api.saveIssue)
    .mockRejectedValueOnce({ code: 'database', message: 'Disk full' })
    .mockResolvedValueOnce(issue);
  render(
    <IssueEditor
      project={project}
      issue={issue}
      onClose={close}
      onOpen={vi.fn()}
      onCreate={vi.fn()}
    />,
  );
  await user.clear(screen.getByLabelText('Title'));
  await user.type(screen.getByLabelText('Title'), 'Edited');
  await user.click(screen.getByLabelText('Bug'));
  await user.click(screen.getByRole('button', { name: 'Save changes' }));
  expect((await screen.findByRole('alert')).textContent).toBe('Disk full');
  expect((screen.getByLabelText('Title') as HTMLInputElement).value).toBe(
    'Edited',
  );
  expect((screen.getByLabelText('Bug') as HTMLInputElement).checked).toBe(true);
  expect(close).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Save changes' }));
  expect(close).toHaveBeenCalledOnce();
});
it('shows subtasks and activity and creates a scoped child', async () => {
  const user = userEvent.setup();
  const create = vi.fn(),
    open = vi.fn();
  const child = {
    ...issue,
    id: 'child',
    identifier: 'F1-2',
    title: 'Smaller step',
    parentId: issue.id,
  };
  vi.mocked(api.subtasks).mockResolvedValue({
    ...emptyPage,
    issues: [child],
    total: 1,
  });
  vi.mocked(api.activity).mockResolvedValue({
    events: [
      {
        id: 'a',
        kind: 'issue.created',
        summary: 'Issue created',
        createdAt: issue.createdAt,
      },
    ],
    total: 1,
  });
  const { unmount } = render(
    <IssueEditor
      project={project}
      issue={issue}
      onClose={vi.fn()}
      onOpen={open}
      onCreate={create}
    />,
  );
  await user.click(await screen.findByRole('button', { name: /Smaller step/ }));
  expect(open).toHaveBeenCalledWith(child);
  await user.click(screen.getByRole('button', { name: 'Add subtask' }));
  expect(create).toHaveBeenCalledWith(issue);
  expect(screen.getByText('Issue created')).toBeTruthy();
  unmount();
  render(
    <IssueEditor
      project={project}
      parent={issue}
      onClose={vi.fn()}
      onOpen={vi.fn()}
      onCreate={vi.fn()}
    />,
  );
  await user.type(screen.getByLabelText('Title'), 'Subtask');
  await user.click(screen.getByRole('button', { name: 'Create issue' }));
  expect(api.saveIssue).toHaveBeenCalledWith(
    null,
    expect.objectContaining({
      parentId: issue.id,
      projectId: project.id,
      title: 'Subtask',
    }),
  );
});
it('requires confirmation for deletion and discard; archive uses scoped IPC', async () => {
  const user = userEvent.setup(),
    close = vi.fn();
  vi.mocked(api.deleteIssue).mockResolvedValue();
  vi.mocked(api.archiveIssue).mockResolvedValue();
  const { unmount } = render(
    <IssueEditor
      project={project}
      issue={issue}
      onClose={close}
      onOpen={vi.fn()}
      onCreate={vi.fn()}
    />,
  );
  await user.click(screen.getByRole('button', { name: 'Delete issue' }));
  expect(api.deleteIssue).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Keep editing' }));
  await user.click(screen.getByRole('button', { name: 'Archive issue' }));
  expect(api.archiveIssue).toHaveBeenCalledWith(project.id, issue.id, true);
  expect(close).toHaveBeenCalledOnce();
  unmount();
  close.mockClear();
  render(
    <IssueEditor
      project={project}
      issue={issue}
      onClose={close}
      onOpen={vi.fn()}
      onCreate={vi.fn()}
    />,
  );
  await user.type(screen.getByLabelText('Title'), ' change');
  await user.click(screen.getByRole('button', { name: 'Cancel' }));
  expect(close).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Keep editing' }));
  await user.click(screen.getByRole('button', { name: 'Delete issue' }));
  await user.click(screen.getByRole('button', { name: 'Delete permanently' }));
  expect(api.deleteIssue).toHaveBeenCalledWith(project.id, issue.id);
  expect(close).toHaveBeenCalledOnce();
});
it('queries the backend for search, status, priority, sorting, archive and pagination', async () => {
  const user = userEvent.setup();
  vi.mocked(api.issues).mockResolvedValue({
    ...emptyPage,
    issues: [issue],
    total: 30,
    projectTotal: 30,
    done: 2,
  });
  render(<ProjectIssues project={project} onBack={vi.fn()} onEdit={vi.fn()} />);
  await screen.findByRole('button', { name: /F1-1/ });
  await user.type(screen.getByLabelText('Search issues'), 'engine');
  await user.selectOptions(
    screen.getByLabelText('Filter issue status'),
    'todo',
  );
  await user.selectOptions(
    screen.getByLabelText('Filter issue priority'),
    'high',
  );
  await user.selectOptions(screen.getByLabelText('Sort issues'), 'priority');
  await waitFor(() =>
    expect(api.issues).toHaveBeenLastCalledWith(
      expect.objectContaining({
        search: 'engine',
        status: 'todo',
        priority: 'high',
        sort: 'priority',
        offset: 0,
      }),
    ),
  );
  await user.click(screen.getByRole('button', { name: 'Next' }));
  await waitFor(() =>
    expect(api.issues).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: 25 }),
    ),
  );
  await user.click(screen.getByLabelText('Archived'));
  await waitFor(() =>
    expect(api.issues).toHaveBeenLastCalledWith(
      expect.objectContaining({ archived: true, offset: 0 }),
    ),
  );
});
it('shows a retryable read error, then recovers with a real empty state', async () => {
  const user = userEvent.setup();
  vi.mocked(api.issues)
    .mockRejectedValueOnce({ message: 'Unavailable' })
    .mockResolvedValue(emptyPage);
  render(<ProjectIssues project={project} onBack={vi.fn()} onEdit={vi.fn()} />);
  expect((await screen.findByRole('alert')).textContent).toContain(
    'Unavailable',
  );
  await user.click(screen.getByRole('button', { name: 'Retry issues' }));
  expect(await screen.findByText('A clear path starts here.')).toBeTruthy();
  await user.keyboard('n');
  expect(screen.getByRole('dialog').textContent).toContain('New issue');
});
it('creates, edits and confirms deletion of workspace labels', async () => {
  const user = userEvent.setup();
  vi.mocked(api.saveLabel).mockResolvedValue([label]);
  vi.mocked(api.deleteLabel).mockResolvedValue([]);
  render(<LabelsDialog workspaceId={project.workspaceId} onClose={vi.fn()} />);
  await user.type(screen.getByLabelText('Label name'), 'Design');
  await user.click(screen.getByRole('button', { name: 'Create label' }));
  expect(api.saveLabel).toHaveBeenCalledWith(null, {
    workspaceId: project.workspaceId,
    name: 'Design',
    color: '#8b5cf6',
  });
  await user.click(screen.getByRole('button', { name: 'Edit Bug' }));
  await user.clear(screen.getByLabelText('Label name'));
  await user.type(screen.getByLabelText('Label name'), 'Defect');
  await user.click(screen.getByRole('button', { name: 'Save label' }));
  expect(api.saveLabel).toHaveBeenCalledWith(
    label.id,
    expect.objectContaining({ name: 'Defect' }),
  );
  await user.click(screen.getByRole('button', { name: 'Delete Bug' }));
  expect(api.deleteLabel).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Delete permanently' }));
  expect(api.deleteLabel).toHaveBeenCalledWith(project.workspaceId, label.id);
});
it('renders Markdown while rejecting HTML, executable links, images and navigation', () => {
  const { container } = render(
    <MarkdownPreview
      text={
        '# Hello\n\n**Bold**\n\n<script>alert(1)</script>\n\n[bad](javascript:alert)\n\n![remote](https://example.com/a.png)\n\n[safe](https://example.com)'
      }
    />,
  );
  expect(screen.getByRole('heading', { name: 'Hello' })).toBeTruthy();
  expect(container.querySelector('strong')?.textContent).toBe('Bold');
  expect(container.querySelector('script,img,a,iframe')).toBeNull();
  expect(screen.getByText('[Image: remote]')).toBeTruthy();
});
it('ignores an old project response after navigation and serializes mutations', async () => {
  let complete!: (page: typeof emptyPage) => void;
  vi.mocked(api.issues)
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          complete = resolve;
        }),
    )
    .mockResolvedValueOnce({ ...emptyPage, total: 4 });
  const first = useIssueStore.getState().load(defaultQuery('old'), 'w');
  await useIssueStore.getState().load(defaultQuery('new'), 'w');
  complete({ ...emptyPage, total: 99 });
  await first;
  expect(useIssueStore.getState().page.total).toBe(4);
  let finish!: () => void;
  const mutation = useIssueStore.getState().mutate(
    () =>
      new Promise<void>((resolve) => {
        finish = resolve;
      }),
  );
  const duplicate = vi.fn();
  expect(await useIssueStore.getState().mutate(duplicate)).toBeNull();
  expect(duplicate).not.toHaveBeenCalled();
  finish();
  await mutation;
});

it('retains label drafts after a rejected save', async () => {
  const user = userEvent.setup();
  vi.mocked(api.saveLabel).mockRejectedValue({
    code: 'validation',
    message: 'A label with this name already exists.',
  });
  render(<LabelsDialog workspaceId={project.workspaceId} onClose={vi.fn()} />);
  await user.type(screen.getByLabelText('Label name'), 'Bug');
  await user.click(screen.getByRole('button', { name: 'Create label' }));
  expect((await screen.findByRole('alert')).textContent).toContain(
    'already exists',
  );
  expect((screen.getByLabelText('Label name') as HTMLInputElement).value).toBe(
    'Bug',
  );
});

it('treats a committed save as successful even when the following refresh fails', async () => {
  const user = userEvent.setup();
  const close = vi.fn();
  useIssueStore.setState({ query: defaultQuery(project.id) });
  // The parent's read succeeds; the store refresh after the write fails.
  vi.mocked(api.issues)
    .mockResolvedValueOnce(emptyPage)
    .mockRejectedValueOnce({ message: 'Read unavailable' });
  render(
    <IssueEditor
      project={project}
      onClose={close}
      onOpen={vi.fn()}
      onCreate={vi.fn()}
    />,
  );
  await user.type(screen.getByLabelText('Title'), 'Committed');
  await user.click(screen.getByRole('button', { name: 'Create issue' }));
  expect(close).toHaveBeenCalledOnce();
  expect(api.saveIssue).toHaveBeenCalledOnce();
  expect(useIssueStore.getState()).toMatchObject({
    error: 'Read unavailable',
    mutationError: null,
    busy: false,
  });
});
