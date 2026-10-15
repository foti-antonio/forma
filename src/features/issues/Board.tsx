import { useRef, useState } from 'react';
import {
  DndContext,
  PointerSensor,
  KeyboardSensor,
  useSensor,
  useSensors,
  useDroppable,
  closestCorners,
  type DragEndEvent,
} from '@dnd-kit/core';
import {
  SortableContext,
  useSortable,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { GripVertical } from 'lucide-react';
import {
  type Issue,
  type IssueStatus,
  type IssuePage,
  type Label,
} from '../../lib/ipc';
import { useIssueStore, emptyPage, moveBoardIssue } from './store';
import { statuses, priorities } from './options';
import { Button } from '../../components/ui/button';

const statusKeys = Object.keys(statuses) as IssueStatus[];
export function Board({ onOpen }: { onOpen: (issue: Issue) => void }) {
  const { columns, labels, busy, loading, offsets, columnPage, mutationError } =
    useIssueStore();
  const [announcement, announce] = useState('');
  const moving = useRef(false);
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );
  const move = async (
    issue: Issue,
    status: IssueStatus,
    before: string | null,
    after = false,
  ) => {
    if (moving.current || busy || loading) return;
    moving.current = true;
    const focusBeforeMove = document.activeElement;
    announce('');
    const result = await moveBoardIssue(issue, status, before, after);
    announce(
      result
        ? `${issue.identifier} moved to ${statuses[status]}.`
        : 'Move failed. The board has been refreshed.',
    );
    moving.current = false;
    // Restore card focus after the canonical refresh, or the column when filtered out.
    requestAnimationFrame(() => {
      // A later user action owns focus even if this frame was delayed.
      if (
        document.activeElement !== focusBeforeMove &&
        document.activeElement !== document.body
      )
        return;
      (
        document.getElementById('card-' + issue.id) ??
        document.getElementById('column-' + status)
      )?.focus();
    });
  };
  const keyboardMove = (issue: Issue, key: string) => {
    const page = columns[issue.status] ?? emptyPage;
    const index = page.issues.findIndex((i) => i.id === issue.id);
    if (key === 'ArrowUp' && index > 0)
      void move(issue, issue.status, page.issues[index - 1].id);
    if (key === 'ArrowDown' && index < page.issues.length - 1)
      void move(issue, issue.status, page.issues[index + 1].id, true);
    const statusIndex = statusKeys.indexOf(issue.status);
    if (key === 'ArrowLeft' && statusIndex > 0)
      void move(issue, statusKeys[statusIndex - 1], null);
    if (key === 'ArrowRight' && statusIndex < statusKeys.length - 1)
      void move(issue, statusKeys[statusIndex + 1], null);
  };
  const dropped = ({ active, over }: DragEndEvent) => {
    if (!over || active.id === over.id) return;
    const issues = Object.values(columns).flatMap((p) => p.issues);
    const issue = issues.find((i) => i.id === active.id);
    const target = issues.find((i) => i.id === over.id);
    const status = target?.status ?? statusKeys.find((s) => s === over.id);
    if (issue && status) {
      // Dropping onto a later sibling inserts after it; other drops insert before.
      const siblings = columns[status]?.issues ?? [];
      const from = siblings.findIndex((i) => i.id === issue.id);
      const to = siblings.findIndex((i) => i.id === over.id);
      const after = from >= 0 && to > from;
      void move(issue, status, target?.id ?? null, after);
    }
  };
  return (
    <>
      <p className="muted board-help" id="board-help">
        Enter opens an issue. Alt + ↑ / ↓ reorders visible cards; Alt + ← / →
        moves to the end of an adjacent column. On a drag handle, Space picks
        up, arrows move, Space drops and Escape cancels. Each column shows up to
        25 matches.
      </p>
      <p role="status" aria-live="polite" className="board-announcement">
        {announcement}
      </p>
      {mutationError && (
        <p role="alert" className="error-banner">
          {mutationError}
        </p>
      )}
      <DndContext
        sensors={sensors}
        collisionDetection={closestCorners}
        onDragEnd={dropped}
      >
        <div
          className="kanban-board"
          aria-label="Kanban board"
          aria-describedby="board-help"
          aria-busy={loading}
        >
          {statusKeys.map((status) => (
            <Column
              key={status}
              status={status}
              page={columns[status] ?? emptyPage}
              offset={offsets[status] ?? 0}
              disabled={busy || loading}
              onPage={(offset) => void columnPage(status, offset)}
            >
              <SortableContext
                items={(columns[status]?.issues ?? []).map((i) => i.id)}
                strategy={verticalListSortingStrategy}
              >
                {(columns[status]?.issues ?? []).map((issue) => (
                  <Card
                    key={issue.id}
                    issue={issue}
                    labels={labels}
                    disabled={busy || loading}
                    onOpen={() => onOpen(issue)}
                    onKey={(key) => keyboardMove(issue, key)}
                  />
                ))}
              </SortableContext>
            </Column>
          ))}
        </div>
      </DndContext>
    </>
  );
}
function Column({
  status,
  page,
  children,
  offset,
  disabled,
  onPage,
}: {
  status: IssueStatus;
  page: IssuePage;
  children: React.ReactNode;
  offset: number;
  disabled: boolean;
  onPage: (offset: number) => void;
}) {
  const { setNodeRef, isOver } = useDroppable({ id: status, disabled });
  return (
    <section
      ref={setNodeRef}
      className={'kanban-column' + (isOver ? ' drop-target' : '')}
      aria-label={statuses[status]}
      tabIndex={-1}
      id={'column-' + status}
    >
      <h2>
        <span className={'column-dot status-' + status} />
        {statuses[status]} <span className="muted">{page.total}</span>
      </h2>
      <div className="kanban-cards">
        {children}
        {page.issues.length === 0 && (
          <p className="column-empty">
            {page.total
              ? 'No cards on this page.'
              : 'No matching issues. Drop an issue here to move it.'}
          </p>
        )}
      </div>
      {(page.total > 25 || offset > 0) && (
        <div className="column-pagination">
          <Button
            variant="ghost"
            disabled={disabled || offset === 0}
            onClick={() => onPage(Math.max(0, offset - 25))}
          >
            Previous
          </Button>
          <span>
            {offset + 1}–{Math.min(offset + 25, page.total)} / {page.total}
          </span>
          <Button
            variant="ghost"
            disabled={disabled || offset + 25 >= page.total}
            onClick={() => onPage(offset + 25)}
          >
            Next
          </Button>
        </div>
      )}
    </section>
  );
}
function Card({
  issue,
  labels,
  disabled,
  onOpen,
  onKey,
}: {
  issue: Issue;
  labels: Label[];
  disabled: boolean;
  onOpen: () => void;
  onKey: (key: string) => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: issue.id, disabled });
  return (
    <article
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={'kanban-card' + (isDragging ? ' dragging' : '')}
    >
      <div className="card-meta">
        <span>{issue.identifier}</span>
        <button
          className="drag-handle icon-button"
          aria-label={'Drag ' + issue.identifier}
          disabled={disabled}
          {...attributes}
          {...listeners}
        >
          <GripVertical size={16} />
        </button>
      </div>
      <button
        id={'card-' + issue.id}
        className="card-open"
        disabled={disabled}
        onClick={onOpen}
        onKeyDown={(e) => {
          if (
            e.altKey &&
            !e.ctrlKey &&
            !e.metaKey &&
            e.key.startsWith('Arrow')
          ) {
            e.preventDefault();
            e.stopPropagation();
            onKey(e.key);
          }
        }}
      >
        <strong>{issue.title}</strong>
      </button>
      <div className="issue-row-labels">
        {issue.labelIds.map((id) => {
          const label = labels.find((l) => l.id === id);
          return (
            label && (
              <span className="issue-label" key={id}>
                <span
                  className="label-dot"
                  style={{ background: label.color }}
                />
                {label.name}
              </span>
            )
          );
        })}
      </div>
      <div className="card-meta">
        <span className={'priority-' + issue.priority}>
          {priorities[issue.priority]}
        </span>
        {issue.dueDate && (
          <time dateTime={issue.dueDate}>Due {issue.dueDate}</time>
        )}
        {!!issue.subtaskTotal && (
          <span aria-label="Subtask progress">
            {issue.subtaskDone ?? 0}/{issue.subtaskTotal} subtasks
          </span>
        )}
      </div>
    </article>
  );
}
