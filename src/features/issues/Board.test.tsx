import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Board } from './Board';
import { ProjectIssues } from './ProjectIssues';
import { useIssueStore, emptyPage } from './store';
import { api, type Issue, type Project } from '../../lib/ipc';
import { defaultQuery } from './options';
vi.mock('../../lib/ipc', async (original) => ({
  ...(await original<typeof import('../../lib/ipc')>()),
  api: {
    issues: vi.fn(),
    labels: vi.fn(),
    moveIssue: vi.fn(),
    projectView: vi.fn(),
    setProjectView: vi.fn(),
  },
}));
const project: Project = {
  id: 'p',
  workspaceId: 'w',
  name: 'Engine',
  description: '',
  icon: 'code',
  color: '#112233',
  status: 'active',
  archived: false,
  createdAt: '',
  updatedAt: '',
};
const issue = (id: string, status: Issue['status'] = 'todo'): Issue => ({
  id,
  projectId: 'p',
  identifier: 'F1-' + id,
  number: Number(id),
  title: 'Task ' + id,
  description: '',
  status,
  priority: 'high',
  dueDate: '2026-10-20',
  parentId: null,
  labelIds: ['label'],
  position: 0,
  archived: false,
  createdAt: '',
  updatedAt: '',
  subtaskTotal: 3,
  subtaskDone: 1,
});
const a = issue('1'),
  b = issue('2'),
  c = issue('3', 'done');
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.moveIssue).mockResolvedValue(undefined);
  vi.mocked(api.projectView).mockResolvedValue('list');
  vi.mocked(api.setProjectView).mockResolvedValue(undefined);
  vi.mocked(api.labels).mockResolvedValue([
    {
      id: 'label',
      workspaceId: 'w',
      name: 'Bug',
      color: '#ff0000',
      createdAt: '',
      updatedAt: '',
    },
  ]);
  vi.mocked(api.issues).mockImplementation(async (q) => {
    const issues = [a, b, c].filter((i) => !q.status || i.status === q.status);
    return {
      ...emptyPage,
      issues,
      total: issues.length,
      projectTotal: 3,
      done: 1,
    };
  });
  useIssueStore.setState({
    board: true,
    columns: {
      todo: { ...emptyPage, issues: [a, b], total: 2 },
      done: { ...emptyPage, issues: [c], total: 1 },
    },
    offsets: {},
    query: defaultQuery('p'),
    workspaceId: 'w',
    page: emptyPage,
    labels: [
      {
        id: 'label',
        workspaceId: 'w',
        name: 'Bug',
        color: '#ff0000',
        createdAt: '',
        updatedAt: '',
      },
    ],
    loading: false,
    busy: false,
    error: null,
    mutationError: null,
    generation: 0,
  });
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});
it('renders five columns, counts, labels, due dates, progress, empty states and Enter to open', async () => {
  const open = vi.fn();
  const user = userEvent.setup();
  render(<Board onOpen={open} />);
  expect(screen.getByRole('region', { name: 'Todo' }).textContent).toContain(
    '2',
  );
  expect(screen.getByRole('region', { name: 'Backlog' }).textContent).toContain(
    'No matching issues',
  );
  expect(screen.getAllByText('Bug')).toHaveLength(3);
  expect(screen.getAllByText('Due 2026-10-20')).toHaveLength(3);
  expect(screen.getAllByLabelText('Subtask progress')[0].textContent).toContain(
    '1/3',
  );
  screen.getByRole('button', { name: 'Task 1' }).focus();
  await user.keyboard('{Enter}');
  expect(open).toHaveBeenCalledWith(a);
});
it('moves with Alt arrows using anchors and preserves focus after canonical refresh', async () => {
  const user = userEvent.setup();
  render(<Board onOpen={vi.fn()} />);
  screen.getByRole('button', { name: 'Task 2' }).focus();
  await user.keyboard('{Alt>}{ArrowUp}{/Alt}');
  await waitFor(() =>
    expect(api.moveIssue).toHaveBeenCalledWith('p', '2', 'todo', '1', false),
  );
  await waitFor(() =>
    expect(document.activeElement).toBe(
      screen.getByRole('button', { name: 'Task 2' }),
    ),
  );
  expect(screen.getByText('F1-2 moved to Todo.')).toBeTruthy();
  screen.getByRole('button', { name: 'Task 1' }).focus();
  await user.keyboard('{Alt>}{ArrowRight}{/Alt}');
  await waitFor(() =>
    expect(api.moveIssue).toHaveBeenLastCalledWith(
      'p',
      '1',
      'in_progress',
      null,
      false,
    ),
  );
});
it('rejects overlapping moves and refreshes canonical data after a failed move', async () => {
  let reject!: (error: Error) => void;
  vi.mocked(api.moveIssue).mockImplementation(
    () =>
      new Promise((_, r) => {
        reject = r;
      }),
  );
  render(<Board onOpen={vi.fn()} />);
  const card = screen.getByRole('button', { name: 'Task 1' });
  fireEvent.keyDown(card, { key: 'ArrowRight', altKey: true });
  fireEvent.keyDown(card, { key: 'ArrowRight', altKey: true });
  expect(api.moveIssue).toHaveBeenCalledTimes(1);
  reject(new Error('Disk full'));
  await screen.findByRole('alert');
  expect(screen.getByRole('alert').textContent).toContain('Disk full');
  await waitFor(() => expect(api.issues).toHaveBeenCalledTimes(5));
  expect(screen.getByRole('region', { name: 'Todo' }).textContent).toContain(
    'Task 1',
  );
});
it('does not steal focus when another card is focused before the restoration frame', async () => {
  render(<Board onOpen={vi.fn()} />);
  const frames: FrameRequestCallback[] = [];
  vi.spyOn(window, 'requestAnimationFrame').mockImplementation((callback) => {
    frames.push(callback);
    return frames.length;
  });
  const moved = screen.getByRole('button', { name: 'Task 2' });
  moved.focus();
  fireEvent.keyDown(moved, { key: 'ArrowUp', altKey: true });
  await screen.findByText('F1-2 moved to Todo.');
  await waitFor(() => expect(frames.length).toBeGreaterThan(0));
  const next = screen.getByRole('button', { name: 'Task 1' });
  next.focus();
  frames.forEach((callback) => callback(performance.now()));
  expect(document.activeElement).toBe(next);
  fireEvent.keyDown(next, { key: 'ArrowRight', altKey: true });
  await waitFor(() =>
    expect(api.moveIssue).toHaveBeenLastCalledWith(
      'p',
      '1',
      'in_progress',
      null,
      false,
    ),
  );
});
it('shares combined filters across Board and List and bounds every column page', async () => {
  const user = userEvent.setup();
  render(<ProjectIssues project={project} onBack={vi.fn()} onEdit={vi.fn()} />);
  await waitFor(() => expect(api.issues).toHaveBeenCalled());
  await user.click(screen.getByRole('button', { name: 'Board' }));
  await screen.findByLabelText('Kanban board');
  await user.type(screen.getByLabelText('Search issues'), 'Task');
  await user.selectOptions(
    screen.getByLabelText('Filter issue priority'),
    'high',
  );
  await user.selectOptions(
    screen.getByLabelText('Filter issue label'),
    'label',
  );
  fireEvent.change(screen.getByLabelText('Due from'), {
    target: { value: '2026-10-01' },
  });
  fireEvent.change(screen.getByLabelText('Due through'), {
    target: { value: '2026-10-31' },
  });
  await waitFor(() =>
    expect(api.issues).toHaveBeenLastCalledWith(
      expect.objectContaining({
        search: 'Task',
        priority: 'high',
        labelId: 'label',
        dueFrom: '2026-10-01',
        dueTo: '2026-10-31',
        limit: 25,
        sort: 'position',
      }),
    ),
  );
  await user.click(screen.getByRole('button', { name: 'List' }));
  await waitFor(() =>
    expect(api.issues).toHaveBeenLastCalledWith(
      expect.objectContaining({
        search: 'Task',
        priority: 'high',
        labelId: 'label',
        dueFrom: '2026-10-01',
        dueTo: '2026-10-31',
        limit: 25,
      }),
    ),
  );
  await user.click(screen.getByRole('button', { name: 'Clear all filters' }));
  await waitFor(() =>
    expect(api.issues).toHaveBeenLastCalledWith(defaultQuery('p')),
  );
  expect(api.setProjectView).toHaveBeenCalledWith('p', 'board');
});
it('paginates a large column without growing the retained dataset', async () => {
  useIssueStore.setState({
    columns: { todo: { ...emptyPage, issues: [a, b], total: 100 } },
    offsets: {},
  });
  render(<Board onOpen={vi.fn()} />);
  const todo = screen.getByRole('region', { name: 'Todo' });
  await userEvent.click(within(todo).getByRole('button', { name: 'Next' }));
  await waitFor(() =>
    expect(api.issues).toHaveBeenCalledWith(
      expect.objectContaining({ status: 'todo', offset: 25, limit: 25 }),
    ),
  );
  expect(useIssueStore.getState().columns.todo.issues).toHaveLength(2);
});
// Give jsdom measured rectangles so real dnd-kit keyboard collision detection can run.
function measure() {
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(
    function (this: HTMLElement) {
      const article = this.closest('article');
      const title = article?.querySelector('.card-open')?.textContent;
      const region = this.closest('section')?.getAttribute('aria-label');
      const x =
        region === 'Done'
          ? 900
          : region === 'In Progress'
            ? 600
            : region === 'Todo'
              ? 300
              : 0;
      const y = title === 'Task 2' ? 180 : 50;
      return {
        x,
        y,
        top: y,
        left: x,
        right: x + 220,
        bottom: y + 100,
        width: 220,
        height: 100,
        toJSON: () => ({}),
      };
    },
  );
}
it('uses the real dnd-kit keyboard sensor to pick up, reorder and drop', async () => {
  measure();
  const user = userEvent.setup();
  render(<Board onOpen={vi.fn()} />);
  const handle = screen.getByRole('button', { name: 'Drag F1-1' });
  handle.focus();
  await user.keyboard(' ');
  await waitFor(() =>
    expect(handle.closest('article')?.classList.contains('dragging')).toBe(
      true,
    ),
  );
  await user.keyboard('{ArrowDown}');
  await user.keyboard(' ');
  await waitFor(() =>
    expect(api.moveIssue).toHaveBeenCalledWith('p', '1', 'todo', '2', true),
  );
});
it('uses the real pointer sensor to drop a card into another column', async () => {
  measure();
  class Pointer extends MouseEvent {
    readonly isPrimary = true;
    readonly pointerId = 1;
    readonly pointerType = 'mouse';
  }
  vi.stubGlobal('PointerEvent', Pointer);
  render(<Board onOpen={vi.fn()} />);
  const handle = screen.getByRole('button', { name: 'Drag F1-1' });
  fireEvent.pointerDown(handle, { button: 0, clientX: 310, clientY: 70 });
  fireEvent.pointerMove(document, { clientX: 320, clientY: 80 });
  await waitFor(() =>
    expect(handle.closest('article')?.classList.contains('dragging')).toBe(
      true,
    ),
  );
  fireEvent.pointerMove(document, { clientX: 910, clientY: 70 });
  await new Promise((resolve) => setTimeout(resolve, 50));
  fireEvent.pointerUp(document, { clientX: 910, clientY: 70 });
  await waitFor(() =>
    expect(api.moveIssue).toHaveBeenCalledWith('p', '1', 'done', '3', false),
  );
  vi.unstubAllGlobals();
});
