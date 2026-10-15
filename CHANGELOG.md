# Changelog

Versions follow Semantic Versioning. Prereleases are evaluation builds; schema
compatibility is separately documented in [BACKUP](docs/BACKUP.md).

## [0.1.0-beta.1] — Unreleased

Proposed first public beta. No release tag or public release exists yet.

### Included

- Local-first workspaces, projects, issue editing, labels, subtasks, activity,
  filters and accessible List/Kanban views with keyboard movement.
- Native Git folder connection, branch/commit inspection and confirmed issue links.
- Opt-in Codex/Claude executable authorization, exact-task approval, bounded
  execution, cancellation, persistent history and manual result review.
- Explicit approved task queues, a shared one/two-process budget, independently
  approved tests and conservative retained-worktree cleanup.
- Native SQLite backup/restore, staged validation, recovery snapshots and
  transactional migrations through schema 6. No account, telemetry or sync.
- Light/dark/system appearance, native desktop packaging for Linux x86_64,
  Windows x64, Intel macOS and Apple Silicon macOS.

### Release preparation

- Aligned beta metadata, architecture-specific installer inventories, SHA-256
  checksums and complete-candidate verification.
- Protected signing/notarization preparation and draft-only release automation.
- Installation, troubleshooting, security model and contributor templates.
- Deferred Markdown parser loading until preview is requested.

### Limitations

- Candidate Windows installers are unsigned; macOS uses ad-hoc signing unless
  protected credentials are configured. Trusted public distribution is blocked.
- Interactive installation and desktop QA require native platform validation;
  hosted process/database initialization does not establish rendered UI behavior.
- Git worktrees are not OS sandboxes. Providers and approved tests have the OS
  user's privileges. Whole-provider isolation is unavailable and fails closed.
- Backups are unencrypted, redaction is heuristic, and repository/worktree files
  and provider credentials are excluded. No updater is configured.
