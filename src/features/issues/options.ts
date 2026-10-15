import type { IssueQuery } from '../../lib/ipc';
export const statuses = {
  backlog: 'Backlog',
  todo: 'Todo',
  in_progress: 'In Progress',
  done: 'Done',
  cancelled: 'Cancelled',
};
export const priorities = {
  urgent: 'Urgent',
  high: 'High',
  medium: 'Medium',
  low: 'Low',
  none: 'None',
};
export const defaultQuery = (projectId: string): IssueQuery => ({
  projectId,
  search: '',
  status: null,
  priority: null,
  archived: false,
  sort: 'position',
  offset: 0,
  limit: 25,
});
