# Local backup and recovery

Forma’s sidebar offers **Export backup** and **Restore backup…**. Both use native
file dialogs. All file access belongs to Rust; no file path, SQL or general
filesystem API is accepted from the WebView. The application works offline.

## Export

Choose a destination ending in `.forma-backup`. Forma uses SQLite’s online backup
API to take a consistent snapshot of the open database, including committed WAL
data. It does not copy the live database file. A temporary file in the chosen
directory is validated and synchronized before replacing the selected destination.
Cancellation leaves existing files alone. Exporting over the active database or
its WAL/SHM files is rejected.

The snapshot includes **all** workspaces, projects (including archived records),
issue identifiers/counters, issues, parent relationships, labels, issue-label
relationships, activity, active workspace, appearance and per-project List/Board
preferences. SQLite sequences, indexes and triggers are preserved. In M03 the old
WebView appearance value imports into SQLite on first launch; the cache itself,
unsaved editor drafts and transient search/filter/page state are not backed up.

**Backups are not encrypted.** Anyone with file access can read descriptions and
activity. Choose secure local/removable storage and control access yourself.
There is no upload, telemetry, automatic cloud copy or encryption key.

## Versioned format and compatibility

Format version **1** is a single ordinary SQLite database, with the usual SQLite
file header. It is not a ZIP or an archive and contains no paths to extract.

- `PRAGMA user_version = 3`: supported Forma schema.
- `backup_metadata`: one row with `singleton = 1`, `format_version = 1` and
  an export timestamp in UTC RFC3339.
- All tables, indexes and triggers must exactly match the supported schema SQL.
  Extra objects and changed constraints/triggers are rejected.
- `PRAGMA integrity_check` must return `ok`; `foreign_key_check` must return
  no violations.

M03 restores only format 1 / schema 3 exports. A raw M01/M02 database, a future
schema, a changed schema or an unsupported format version is rejected without
changing current data. Open older live databases normally to migrate them
transactionally, then export with M03. Future backup migrations need explicit
implementation and tests; compatibility is not assumed from a filename.

## Restore and rollback

1. Choose a backup using **Restore backup…**.
2. Rust opens it read-only, pins a read transaction, validates versions, complete
   schema, integrity and foreign keys, and creates a private temporary snapshot.
   Changes to the selected original file cannot change the staged restore.
3. Review the explicit **Replace all local data?** confirmation. Cancel or Escape
   discards staging. Validation alone never replaces current data.
4. On **Replace local data**, Forma first exports a recovery copy of the current
   database next to the live database:
   `forma-recovery-<UUID>.forma-backup`. If this fails, replacement does not start.
5. SQLite’s backup API replaces live database contents as one destination
   transaction. A failed or incomplete copy rolls back. If the process/power
   stops during replacement, SQLite journal/WAL recovery determines the complete
   committed state on next startup; a partially copied database is not committed.
6. Forma reopens its connection, re-enables normal database settings, clears
   issue caches and refreshes workspaces, projects, appearance and navigation.
   If connection reopening fails, it restores the recovery snapshot.
7. A success message displays the recovery copy’s location. Recovery copies are
   retained until you deliberately remove them; M03 has no automatic retention.

Avoid simultaneously running multiple Forma processes during restore. Ordinary
application writes are serialized on the database mutex; the frontend disables
conflicting backup actions and issue moves while their operations are running.

## Recovering the previous data

Open **Restore backup…**, select the recovery file and confirm. This follows the
same validation and recovery process, so the state being replaced is also saved.

If Forma cannot launch, preserve the entire app-data directory first, including
any WAL/SHM files, and keep the recovery file. With every Forma process closed,
move the damaged `forma.sqlite3` and its `-wal` / `-shm` siblings into a
separate recovery directory. Copy a known good recovery snapshot to
`forma.sqlite3` in the original app-data directory, then launch Forma. Never
leave stale WAL/SHM files beside the replacement snapshot, and never edit a live
database manually. See README for OS-specific app-data locations.

Do not delete the original data or recovery copy until the restored workspaces,
projects and issues have been checked. A disk/device failure can destroy both
live data and a recovery copy on that device; keep exports on separate storage
when you need protection against hardware loss.

## Verification boundaries

Rust tests cover complete data roundtrips/reopen, recovery restores, tampered
source after staging, confirmation gating, unsupported schema/format versions,
extra triggers, foreign-key damage, garbage/truncated files, active-database
destination protection and interruption of an actual SQLite backup transaction.
Frontend tests cover selection, confirmation, cancellation, export/restore errors
and cache refresh. Interruption is exercised by ending a partial SQLite copy,
rather than claiming a real power-loss test. See M03.md for actual platform runs.
