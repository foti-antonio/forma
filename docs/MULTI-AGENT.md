# Multi-agent execution and reliability

M06 keeps execution conservative: one process by default, at most two after an
explicit settings change. The budget is shared by agent runs and independently
approved tests. Reducing the limit does not kill existing work; it prevents new
launches until enough processes stop. No task starts on application startup.

## Approval and queue lifecycle

A backend preview captures the exact saved task, project context, authorized
repository/common Git directory/HEAD, provider executable path/SHA-256, permission
policy and runtime/output limits. It expires five minutes after preview creation.
Each preview is single use. Claude additionally requires explicit per-task consent
to executing without a whole-process OS sandbox.

**Approve & run once** preserves M05's immediate execution path. **Approve & add
to queue** persists the snapshot without executing it. Up to 100 tasks may await
execution; each issue has at most one approved queue entry. Queue history is
bounded to the first 100 records per response, ordered with approved tasks first.
Expired entries are invalidated before dispatch. Users can cancel and move an
approved entry earlier; cancellation clears the execution snapshot.

**Start Next** dispatches one eligible approved task if a slot is available.
**Start Approved Queue** freezes membership to the tasks approved at that click,
fills available slots and dispatches remaining members as slots become available.
Later approvals need another Start action. **Stop queue dispatch** disables future
dispatch without cancelling already running processes. Cancel those individually.
Failed dispatch initialization releases the in-memory session so an explicit
retry can proceed. If the dispatcher thread cannot start, already launched runs
continue independently and the remaining queue can be started again.
The dispatcher checks every 250 ms on a Rust thread, sharing the service mutex
with other commands; WebView polling only reads state and never starts tasks.

Queue states: Approved → Launching → Started, or Invalidated/Cancelled. The run
has its own Queued → Running → Completed/Failed/Cancelled lifecycle. Launching
is recorded before launch; a crash at any point invalidates the queue snapshot
and interrupts unfinished runs on restart. Execution is never replayed. A launch
failure records a readable invalidation; partially created artifacts stay retained.
Ordinary restarts preserve queue metadata but invalidate all unused approvals,
requiring fresh previews. An in-memory dispatch session never survives restart.

Immediately before dispatch, Rust reconstructs the task context and revalidates
repository authorization, clean primary tree, common Git directory, HEAD, issue
revision, executable path/digest, CLI capabilities, policy and limits. Changes
require fresh approval. The worker checks executable identity again before spawn.
The queue cannot approve elevation; task text, repository instructions and agent
output are inert input and cannot grant permissions.

## Concurrency and source-control isolation

The service serializes reservations and worktree creation. Each run receives a
UUID branch and app-data `agent-worktrees/<run UUID>` path. A SQLite partial unique
index enforces per-issue run exclusivity. Separate worker connections supervise
process groups/jobs and persist independent logs, progress, exit and review state.
Different issues may use the same repository or different repositories. UUID
branches/paths prevent collisions; creation never overwrites existing artifacts.
Forma never executes a provider in the primary worktree, commits, merges or pushes.

The desktop holds an exclusive app-data lock. This prevents two desktop owners
for one database; the service is not designed as a multi-process server. External
Git clients and malicious same-user processes can still race filesystem changes.
Worktrees share Git objects, refs, configuration and metadata. They are
source-control isolation, never security sandboxes.

## Process security and limits

Only native file selection authorizes executables. Canonical paths and SHA-256
are checked; changed paths or bytes need reauthorization. Windows requires native
`.exe` files. Unix executable files must not be group/world writable. Provider
arguments come from Rust adapters, never frontend shell strings. Provider help
and version checks are bounded and run only for an authorized installation.

Unix process groups plus an owner-pipe watchdog terminate ordinary descendants
on cancellation, timeout, normal leader exit or owner death. Windows uses
kill-on-close Job Objects through command-group. Cancellation before spawn returns
without starting a child. These mechanisms are not a hostile-process sandbox:
Unix descendants can deliberately leave the process group, and platform behavior
requires native validation. Executable hashing cannot eliminate all same-user
filesystem races between inspection and OS execution.

Each run has a 1–7200 second deadline and 1024–1048576 byte output/log budget.
Stdout/stderr share a bounded 32-record channel; individual lines are at most
8 KiB. Raw bytes are counted before redaction, including omitted long lines.
A supervisor drains at most 64 records before checking cancellation, timeout and
output exhaustion. Exceeding the output limit terminates the process group/job
with a specific failure result. Logs are redacted heuristically and persist at
most five times per second. UI history holds one 20-row page and polls at 1 Hz.
A noisy child cannot create an unbounded log queue. Git status/diff work and
authorized provider capability checks can still temporarily delay service calls.

There is no enforced CPU, RAM, disk or network quota. Providers, test runners and
project tests run as the current OS user. Home/authentication files, network and
shared repository metadata may remain accessible. Environment inheritance is an
allowlist; Forma does not pass arbitrary API keys, tokens or loader variables.
Redaction is incomplete and local backups are unencrypted. Provider restrictions
are defense in depth, not a promise of complete containment.

**Require whole-provider OS isolation** fails closed on this build: no supported
whole-process containment implementation is available. Enabling it blocks
preview/launch; Forma never silently substitutes unrestricted execution. Codex
workspace-write restricts its commands. Claude retains its reduced file-tool set,
no shell/MCP tools, restricted mode and separate host-risk consent. Use a separate
OS account/container for stronger whole-process isolation.

## Independently approved tests

In result review, authorize a project test runner through the native executable
picker. Configure a JSON array of arguments, timeout and output limit. Forma
rejects direct command shells and does not interpret shell strings or expand
arguments. A trusted test runner can execute repository code and subprocesses;
this is deliberate user-authorized execution with the stated host exposure.
Selecting a runner or saving arguments does not start tests.

**Save & preview independent tests** displays the exact executable/argument array,
working directory and limits. **Approve & run independent tests** consumes a
five-minute single-use preview. Launch revalidates executable identity, command
configuration, stopped agent state and worktree identity. Tests use the shared
process budget and independent cancellation; one test process per run is allowed.
Exit code, wall duration, bounded output and timestamps persist in SQLite.
Interrupted tests become Interrupted and never restart automatically. An exit
code proves only the outcome of that process, not test completeness or truth of
its output. Provider-reported test claims remain separate, untrusted logs.
Neither agent completion nor tests change an issue to Done.

## Retained worktrees and safe cleanup

**AI agents & runs → Worktree Management** pages through runs, showing issue, run,
repository, branch, path, execution/review/cleanup status, local changes and disk
usage where measurable. Scans stop at 10,000 entries and never follow symlinks;
unavailable sizes and failed inspections are explicit. Open uses the native
folder opener after identity validation. Restored historical paths stay blocked.

Cleanup needs explicit confirmation, a stopped and reviewed run, no running tests,
a registered unlocked linked worktree with the expected UUID path/common Git
directory/branch, and a clean status including untracked and ignored files.
Symlinks or unknown scans block cleanup. Forma never force-removes unknown or
modified worktrees or deletes the primary tree. Preserve changes manually before
cleanup; merely accepting/rejecting a result does not make dirty content disposable.
Branches and historical metadata remain after cleanup.

Cleanup transitions Retained → Removing → Removed/Failed. Restart turns Removing
into Failed. An explicit retry can safely prune leftover registration for a missing
path only after a previous cleanup attempt, without force removal. Missing or
corrupted retained worktrees otherwise require manual inspection. An external
same-user writer can race a clean-tree check; stop other tools before cleanup.

## Backup, validation and distribution

Append-only migration 006 preserves M01–M05 tables and adds scheduler, queue,
cleanup and independent-test data. Format 4/schema 6 snapshots accept exact
released formats 1–3 through staged migrations. Restore strips repository,
provider and test-executable authorization and worktree access, invalidates queue
approvals, interrupts unfinished work and never starts anything. Secrets and
worktree contents are excluded. See [BACKUP](BACKUP.md).

The CI matrix retains Linux, Windows, Intel macOS and Apple Silicon macOS. Each
desktop target runs frontend checks, Rust/production IPC tests, clippy and native
packaging. Simulated native executables exercise concurrency, cancellation and
Git lifecycle on each runner. The package smoke installs Windows NSIS silently,
extracts Linux deb contents, and mounts/copies macOS DMG contents before launching
with isolated data. Initialization and database integrity are distinct from
interactive installer/rendering validation. Windows installers remain unsigned;
macOS ad-hoc signing is not Developer ID signing or notarization. No remote CI
result is implied by adding workflow steps. See [M06](../M06.md) for actual runs,
provider checks, measurements and unverified platforms.
