# Security model — 0.1.0-beta.1

Forma stores project data locally in Rust-owned SQLite. It has no cloud service,
telemetry, authentication service, updater or general JavaScript filesystem,
shell, network or opener API. Enabling a coding provider is optional; provider
inference can use that provider's network and authentication.

## Desktop trust boundary

The single `main` capability has an empty permission list. Tauri application
commands still exist: capability emptiness does not disable Forma's registered
IPC handlers. React → feature store → typed IPC → Rust command → validated service
→ parameterized repository remains the boundary. Rust commands use native dialogs
for backup, repository and executable selection; frontend code cannot choose
arbitrary filesystem paths. Folder opening validates retained worktree identity
before using the native opener. No capability was expanded for M07.

CSP restricts scripts to bundled assets, IPC connections to local Tauri transport,
images to local/data assets, and styles to local plus inline project colors.
Markdown skips raw HTML and displays links/images as inert text. Provider output
is bounded plain text, never active HTML. A renderer or native dependency
vulnerability could still cross these boundaries; keep dependencies patched.

## Executable authorization and approval

Selecting a trusted executable authorizes bounded help/version execution. Native
canonical path and SHA-256 identity are stored and rechecked; changes require
reauthorization. Unix setuid/setgid and group/world-writable files and Windows shell wrappers are
rejected. This establishes selected-file identity, not software publisher trust.
Same-user filesystem races cannot be eliminated by hashing alone.

A task preview expires after five minutes and is single use. Launch revalidates
saved issue/context, repository/HEAD, executable identity, capability flags,
policy and limits. Queue membership needs explicit approval and explicit dispatch;
startup/restore never starts tasks. Tests require their own runner selection,
exact argument/cwd preview and approval. Shell command strings are not interpreted,
but an authorized test runner can execute repository code and subprocesses.

## Host exposure

**Git worktrees are not OS security sandboxes.** They share objects, refs,
configuration and Git metadata. Providers and project tests run as your OS user;
home files, credentials, network and other repositories may be accessible.
Codex command restrictions and Claude's reduced file tools are defense in depth.
Claude requires separate host-risk acknowledgement. Requiring whole-provider OS
isolation fails closed because that implementation is unavailable.

Use trusted repositories/executables and a separate OS account or externally
managed container when stronger isolation is needed. Review output and changes
before manual integration. Forma never automatically commits, merges or pushes.
Prompt injection and hostile providers are not contained by task instructions.

Process groups/watchdogs and Windows kill-on-close jobs stop ordinary descendants
on cancellation, limits and owner exit; deliberately escaped processes are a
residual risk. Time/output budgets and bounded channels do not enforce CPU, RAM,
disk or whole-process network quotas. One process is the default shared budget;
users can explicitly select two. Cleanup requires a stopped, reviewed, known,
clean worktree and confirmation; it never force-removes unknown/dirty trees.

## Privacy and backup

Log/diff redaction is heuristic and incomplete. Private task/code data and queue
snapshots may enter SQLite, logs and exports. Backups are **not encrypted**.
Provider credentials, repository objects and worktree files are excluded; back up
those separately. Provider-owned logs and credential storage remain provider-owned.
Do not attach private databases or raw logs to public bug reports.
Restore validates exact schemas and integrity in private staging, creates recovery
copies and strips repository/executable/worktree authorization. Historical runs
stay readable, unfinished work is interrupted and approvals are invalidated.
Recovery copies and exported backups have no automatic expiration or pruning;
review them after a verified restore and deliberately remove obsolete copies from
secure storage. Deleting a workspace does not erase historical exported backups.
See [BACKUP](BACKUP.md) and [AGENTS-INTEGRATION](AGENTS-INTEGRATION.md).

## Package trust

SHA-256 checksums detect changed bytes; they do not authenticate a publisher.
Unsigned Windows and ad-hoc macOS candidates are not trusted public packages.
Protected Authenticode signing and Developer ID/notarization plus native trust
checks are prerequisites for unrestricted distribution. No security bypass is
part of installation guidance. See [RELEASING](RELEASING.md) and [M07](../M07.md)
for the actual status. Report vulnerabilities through [SECURITY](../SECURITY.md).
