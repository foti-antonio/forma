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

## M05 — AI agent integration

Implemented explicitly approved Codex CLI and Claude Code adapters, native
executable authorization, saved-issue prompt previews, dedicated Git worktrees,
Rust process supervision, bounded redacted progress/logs, cancellation/timeouts,
persistent lifecycle/history and human result review. Migration 005 and format-3
backups preserve validated M03/M04 compatibility and strip imported path access.
See [AGENTS-INTEGRATION](AGENTS-INTEGRATION.md) and [M05](../M05.md) for execution
policies, actual verification and platform limitations.

## M06 — Multi-agent execution and reliability

Implemented a default-one, explicitly configurable maximum-two process budget,
persistent approval queue with explicit dispatch and restart invalidation,
independent per-run cancellation/logs/review, separately approved project tests
and confirmed cleanup of known clean reviewed worktrees. Migration 006 and
format-4 backups preserve earlier data and remove restored path authorizations.
The four-platform CI matrix now includes native lifecycle tests and packaged
initialization smoke steps. See [MULTI-AGENT](MULTI-AGENT.md) for scheduler/security
rules and [M06](../M06.md) for actual checks and platform gaps.

## M07 — Recommended release scope

1. Execute and inspect the hosted Windows and both macOS native matrices, then
   manually exercise interactive installers and rendering on clean machines.
2. Developer ID/notarization and Authenticode release signing with protected
   credentials, plus authenticated real Claude execution on an existing login.
3. Stronger whole-provider OS containment and hostile descendant containment;
   brokered tool approvals without provider self-elevation.
4. Real-provider concurrent CPU/RAM/disk measurements, large-repository stress,
   configurable backup recovery retention and encryption with defined key handling.
5. Generated IPC contracts and larger frontend bundle splitting.

Cloud synchronization and Forma authentication remain out of scope.
