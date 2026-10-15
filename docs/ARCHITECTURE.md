# Architecture

Forma is a single-window Tauri 2 desktop application. React renders the shell;
feature forms handle workspace, project, issue and label inputs. Zustand holds the most recent
authoritative database snapshot plus loading, mutation, and error state.

## Boundaries

```text
React features and shared UI
    → Zustand mutation coordinator
    → src/lib/ipc.ts (typed invoke wrapper)
    → Rust commands.rs (IPC arguments / structured results)
    → service.rs (validation, use cases, transaction ownership)
    → repository.rs + migrations/ (parameterized SQLite, row mapping)
    → OS application-data directory / forma.sqlite3
```

Snapshot reads contain all workspaces, active workspace selection, and projects
only for the selected workspace. Mutation responses return refreshed snapshots.
The store serializes mutations and prevents reads from racing mutations. Failed
writes preserve the last good snapshot and form inputs; the user can retry.
Transient dialogs and combined filters are React state. Active workspace,
appearance and project view selection are SQLite state; WebView local storage
caches appearance and supplies the first-launch M02 preference import.

Short commands retain the original mutex around one SQLite connection. Issue
listing, board movement and native backup commands run on Tauri’s async runtime,
keeping SQLite and file work off the WebView event thread. They still serialize
database access through the mutex; no second persistence implementation exists. IPC types are currently maintained manually on each side; frontend
contract tests, Rust validation tests and an opt-in Tauri IPC transport test cover behavior, but generated bindings
are a future improvement.

## Database

SQLite is compiled into Rust through rusqlite. Startup enables foreign keys,
a five-second busy timeout, and WAL, then runs the initial SQL migration inside
a transaction and sets its schema version. Migration 002 upgrades issues, allocates project
keys, backfills per-project numbers deterministically, and adds relationship
constraints and activity lookup indexes. Each migration and its version update
commit together. Startup skips applied versions and rejects schemas newer than
version 3. Migration 003 adds project view state, backup format metadata, nullable appearance
state (null imports the M02 preference), and a measured board-order index.
Migrations 001 and 002 are unchanged; tests upgrade populated v1 fixtures and
verify that a failed upgrade rolls back and can be retried.

Workspaces own projects, labels, and activity. Projects own issues. Issue-label
joins cascade on either parent deletion. Project deletion retains a workspace
activity record with a null project reference. Workspace deletion intentionally
removes its entire history. Project create/edit/archive/delete and workspace
create/rename log activity atomically with their changes.

All entities use UUID v4 keys and UTC RFC3339 timestamps. Join rows use the pair
of entity keys and a creation timestamp. Foreign-key and ordering/filter columns
are indexed. The singleton app_state table owns the active workspace reference;
deletion selects the oldest remaining workspace or null.

Workspace names are trimmed, 1–80 characters. Project names are trimmed,
1–100 characters; control characters are rejected. Descriptions allow up to
4,000 characters. Icons/statuses are enums validated by the backend and colors
must be six-digit hex values. Project edits cannot move workspaces.

Issue queries use explicit project scopes, parameterized predicates, allowlisted
sort clauses and bounded offset pagination. Typed issue/label commands keep
workspace and project identities in each mutation. Rust validates every input,
parent relationship and label assignment. Database triggers add defense for
hierarchy and workspace joins. Single-level subtasks survive parent deletion.
See [ISSUES](ISSUES.md) for the complete domain policy.

The issue store owns list, labels, loading, read errors and mutation errors;
request generations prevent old project/filter responses replacing newer data.
It serializes writes and refreshes the list after successful mutations. A refresh
failure remains a separate retryable read error after a committed save, avoiding
duplicate creation. Editor drafts are local React state, retained on write errors.
Detail reads pass through the issue store module and use cancellation guards.
Children and activity each paginate in groups of 20. Activity contains field names
and action summaries rather than issue content; writes/events commit atomically.
Project key allocation and per-project counters make identifiers independent of
renames, edits, deletion, archive and restart.

## UI

The shell uses a restrained violet accent, responsive card grid, native title
bar, project navigation, workspace selector, archive, search and status filters.
Issue details use a responsive list, status/priority/search/archive filters,
project counters, keyboard shortcuts and accessible create/edit dialogs. Markdown
preview uses react-markdown with raw HTML disabled, text-only links and alt-only
images; no external content loads. Local Button and Dialog components follow shadcn/ui composition (Radix Slot,
CVA, clsx/tailwind-merge, Radix Dialog); source is owned in the repository.
Tailwind 4 is integrated via Vite; most visual styling uses shared CSS variables
in styles.css. Appearance respects light/dark/system including live changes.

Radix Dialog provides focus trapping, focus restoration, Escape handling and
accessible title/description wiring. Forms use visible labels, native constraints,
and backend validation. Mutations disable conflicting actions and show status;
deletions explain cascade consequences in confirmation dialogs. Minimum native
window size is 760×540; content scrolls and cards reflow.

## Security and builds

No SQL, filesystem, shell, opener, dialog or HTTP plugins are exposed. Rust’s
rfd native dialogs return paths only to backend code; the WebView can request
export, stage, cancel or explicitly confirm restore, but cannot supply paths.
No additional Tauri capabilities were added. The main capability
has no core/plugin permissions because native window decorations supply window
controls. Tauri app commands are the only application API. CSP permits bundled
assets, inline styles for project colors, and local IPC; remote scripts/frames
are not used. There is no external content, telemetry, login, or network client.

Rust domain tests can compile with `--no-default-features`; this excludes the
Tauri host, not the database or business logic. Desktop is the default feature.
CI separately compiles/tests the full host on every target OS, then bundles.
GitHub release preparation reuses CI, checks tag/version agreement, and creates
a draft. Ordinary builds use unsigned Windows packages and ad-hoc macOS signing.

## Verification boundaries

`--no-default-features` exercises real SQLite and services without GTK/WebKit.
`--features ipc-tests` additionally sends JSON through Tauri’s test transport and
the production command handler, including real database reopen verification. Its
mock runtime does not prove native WebView rendering; a separate Linux UI smoke
script exercises the built desktop app. CI runs the IPC test on desktop targets.
See M03.md for commands actually executed and platform limitations.

## M03 board and recovery

The issue store loads five bounded queries (25 rows each) through the same
validated IssueQuery as List. Each column replaces its current page rather than
accumulating an unbounded array. Generation checks discard stale responses.
View preferences pass through the issue store boundary. Card reads include
indexed subtask counts and labels. Board sort is always position/number/UUID.

Keyboard movement was implemented as a canonical anchor operation before pointer
sensors were attached. Both Alt-arrow movement and dnd-kit PointerSensor /
KeyboardSensor use the same mutation coordinator. One in-flight move is allowed;
controls disable until a canonical refresh. Moves use IMMEDIATE transactions,
validate project/archive/status anchors, normalize entire source/destination
columns and log activity in one commit. An anchor may insert before or after an
issue; null appends. Hidden filtered cards retain relative order. Failed moves
roll back and refresh; no optimistic arrays need reconciliation.

SQLite’s backup API exports the entire database into a temporary file, marks
format v1, validates it, synchronizes the file and persists it at the native
selection. Restore validates full schema SQL, format/schema versions, integrity
and foreign keys in a pinned read transaction, then snapshots to private staging.
Confirmation creates a recoverable snapshot of current data and uses SQLite’s
atomic backup destination transaction. Incomplete copying rolls back, including
WAL crash recovery. Connections reopen after replacement; an open failure restores
the recovery snapshot. Frontend generations, issue caches, app snapshot and
project navigation reset after success. See BACKUP.md for operational details.

At 10,000 issues, EXPLAIN showed a temporary sort for each status column.
The new (project_id, archived, status, position, number, id) index removes that
sort. Empty search avoids lowercasing descriptions. Materialized ranking joined
with UPDATE FROM avoids repeated correlated scans during normalization.
Literal substring searches remain linear; no speculative search/index rewrite
was introduced. See M03.md for actual timings and measurement conditions.
