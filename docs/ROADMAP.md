# Roadmap

## M01 — Foundation and desktop app

Standalone Tauri shell, workspace and project CRUD, Rust-owned SQLite,
migrations, active-workspace persistence, themes, validation, tests, CI,
unsigned desktop packaging, and contributor documentation.
See M01.md for verified results and platform limitations.

## M02 — Issue management

Implemented issue CRUD, stable per-project numbering, priorities, due dates,
manual ordering, archive/restore, single-level subtasks, workspace labels,
search/status/priority filters, accessible Markdown editor, paginated lists and
activity, and migration 002 with populated-upgrade/rollback tests. Typed IPC and
real SQLite tests cover isolation and restart persistence. See M02.md for actual
validation, and [ISSUES](ISSUES.md) for domain decisions.

## M03 — Kanban and productivity

Implemented persistent List/Board selection, five status columns, keyboard-first
atomic movement, dnd-kit pointer/keyboard dragging, combined label/date/text/
status/priority filters, bounded per-column pagination and subtask progress.
Versioned SQLite snapshot export/validated restore uses native dialogs, explicit
confirmation, recovery snapshots and transactional replacement. Migration 003
preserves M01/M02 data. Representative 10k-issue measurements justify a board
ordering index and query improvements. See M03.md for actual test, performance
and platform evidence and BACKUP.md for compatibility/recovery.

## M04 — Local Git integration

Implemented native-folder repository connection, read-only local libgit2 status,
branch search and bounded commit history. Issues support confirmed branch/commit
associations, exact-identifier suggestions and historical snapshots surviving
branch deletion, disconnect and restart. Append-only migration 004 and backup
format 2 preserve prior data; M03 exports upgrade in private staging and restored
paths require explicit folder selection before access. See [GIT](GIT.md) and
[M04](../M04.md) for design, actual checks and platform limits.

## M05 — Recommendations

1. Exercise native Windows and both macOS architectures interactively, including
   Git folder selection, worktrees, missing paths, installers and signing.
2. Measure larger real repositories; use evidence to add cancellation, independent
   Git workers, cursor history and status scan resource limits.
3. Consider richer issue Git navigation, historical repository management and
   generated IPC contracts. Any write/network Git requires a separate design.
4. Evaluate FTS, bulk operations, richer Markdown, backup retention and encryption
   with a deliberate key-management model.

Cloud sync, authentication and AI agents remain outside M04.
