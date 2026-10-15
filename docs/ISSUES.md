# Issue domain (M03)

- UUIDs are internal keys. Display identifiers are `F<project key>-<issue number>`,
  for example `F12-7`. Project keys are allocated globally by an AUTOINCREMENT
  table; issue numbers increase per project in the same transaction as creation.
  Neither is derived from an editable name. Deletion never reuses issued numbers.
- Status is Backlog, Todo, In Progress, Done or Cancelled. Priority is Urgent,
  High, Medium, Low or None. New UI issues default to Todo / None.
- Titles are trimmed, 1–200 characters, without control characters. Descriptions
  allow 50,000 characters of CommonMark. Dates are optional calendar dates in
  `YYYY-MM-DD` format, without a time zone. Timestamps are UTC RFC3339.
- Each issue has a nonnegative integer position (UI/backend maximum 1 billion).
  Manual order sorts by position, then issue number and UUID. Priority order is
  Urgent → High → Medium → Low → None, then manual order. Due-date order puts
  unset dates last. Creation/update sorts use descending timestamps and UUID
  tie breakers. List pages are limited to 100 rows; the UI uses 25.
- Search treats input literally, matching title, description and identifier with
  SQLite's ASCII case folding. Status, priority, one workspace label, inclusive due-from/due-through dates,
  and literal search combine with AND in both List and Board. Missing dates
  do not match ranges; No due date is exclusive with ranges. Invalid/reversed
  ranges and foreign-workspace labels are rejected.
  Counters cover the project's selected archive scope; matching count includes
  filters. Offset pagination is deterministic for a fixed dataset; edits between
  requests can shift page boundaries.
- A subtask has one parent in the same project. Parents must be top-level issues;
  issues with children cannot become subtasks. Self-parenting, cycles, and deeper
  nesting are rejected in Rust and database triggers. Removing a parent does not
  delete children: children become top-level and receive activity records.
- Archiving affects only the selected issue. Children retain their archive state
  and remain visible/editable. Existing children can keep an archived parent;
  new children cannot be assigned to it. Restoring preserves the identifier.
- Workspace labels have trimmed names (1–40 characters), six-digit hex colors,
  UUIDs and UTC timestamps. New/renamed names reject ASCII case-insensitive
  duplicates. Legacy M01 names and assignments survive upgrade. A label may be
  assigned once per issue and only within its workspace. Label deletion removes
  assignments, updates affected issue timestamps and records activity atomically.
- Issue creation, actual field changes, archive/restore, deletion and relationship
  changes produce transactional activity. No-op edits do not create events.
  Summaries list changed field names, never description/title contents or old
  values. Deleted issue UUIDs remain in history without a foreign key so history
  can survive deletion. Workspace deletion removes all its history.
- Markdown renders through react-markdown without raw HTML. Links render as text
  and images as alt text, so preview cannot navigate the WebView or fetch remote
  resources. CommonMark supports headings, emphasis, lists, quotes and code;
  GFM tables/task-checkbox extensions and attachments are future work.

- Board pages contain at most 25 matching cards per status (125 retained across
  five columns); the same backend endpoint caps all pages at 100. Pagination
  replaces a column page. Column counts include all matches, not just visible
  cards. List retains its 25-row pagination and all M02 sort options.
- Board movement inserts before/after an issue anchor in the same project,
  status and archive scope; a null anchor appends to that complete column.
  Keyboard up/down crosses the neighboring visible card, including under filters.
  Hidden cards retain their relative order; the operation can cross hidden cards.
  Left/right appends to the adjacent status column. Pagination boundaries require
  changing page; movement within the last visible pair uses an after-anchor so
  it does not accidentally append beyond the unseen next page.
- A move transaction normalizes complete affected columns to dense zero-based
  positions, preserving deterministic position/number/UUID ties, and updates
  status, timestamp and activity atomically. M02 editor numeric order remains
  supported; its ties are deterministic and the next move normalizes them.
  Archive scopes remain independent. Concurrent connections serialize through an
  IMMEDIATE transaction; the UI allows one move at a time and refreshes on failure.
- Cards show stable identifier, title, priority, labels, due date and completed /
  total direct subtask count (including archived children, matching the editor).
  Enter opens details; Alt-arrow movement does not run in text fields.
