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

## M04 — Recommended

1. Exercise Windows and both macOS native file dialogs, installer/launch behavior,
   signing and notarization. Maintain the hosted matrix exercised for M03; build
   results and interactive runtime verification are separate gates.
2. Measure larger/real datasets; evaluate FTS for literal search, cursor paging,
   cancellation and dedicated database workers using measured needs.
3. Add richer Markdown, bulk operations, multiple label selection and optional
   generated IPC contracts after the standalone workflow is stable.
4. Design forward backup migrations and a backup retention UI. Current format
   accepts only the explicitly supported schema; encryption needs a separate
   threat model and key-management design.

Cloud sync, authentication and AI agents remain outside M03.
