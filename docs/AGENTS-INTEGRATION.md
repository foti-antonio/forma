# AI agent integration (M05–M08)

Forma launches locally installed Codex CLI or Claude Code after a user previews
and explicitly approves a saved issue. Provider inference may require network
access. Forma itself has no cloud service, login or API-key database.

## Usage

1. Open **AI agents & runs → Agent settings → Set up AI**. Choose Codex or Claude
   Code, find an installation and authorize only trusted software. Discovery does
   not execute candidates; authorization permits version/help and auth-status
   checks. Follow official installation guidance and rescan if no installation is
   found. A native file picker remains available for custom installations.
2. Sign in using the explicit provider browser-login action, then check
   authentication. Forma never requests passwords, API keys or tokens. Unknown
   status blocks guided readiness; terminal login remains a fallback.
3. Save the issue and choose **Run with AI**. The wizard connects its Git folder
   with the native picker when needed. Preserve local changes yourself; Forma
   rejects dirty repositories. Review conservative permissions and host exposure.
   Recommended settings are explicit and preserve required OS isolation.
4. **Preview first task** opens the existing exact-prompt approval dialog. Review
   the canonical executable/version, repository/base, limits and warning. Claude
   requires its separate host-risk acknowledgement on every run. **Approve & run
   once** starts one task; **Approve & add to queue** records approval without
   execution. Advanced users can still expand **AI Agent → Preview agent task**.
5. Watch progress, elapsed time and bounded logs in issue history or the global
   **Agent runs** panel. **Cancel execution** terminates the process group/job.
6. **Inspect result** shows changed paths, additions/deletions and a bounded text
   diff. Open the worktree to verify results and run tests independently. Accept
   or reject records a review decision. Integrate manually using your Git tools.

## Provider architecture and permissions

React → agent feature coordinator → typed IPC → Rust command → agent service →
SQLite repository/provider supervisor. Blocking commands execute away from the
WebView. Each supervisor owns a separate SQLite connection. No filesystem, shell
or network plugin is exposed to JavaScript; Tauri capabilities remain unchanged.

Native file selection or explicit authorization of a Rust-discovered candidate authorizes a CLI. PATH discovery is informational and
never executes a candidate. Authorization stores a canonical path and SHA-256
identity. A changed executable needs fresh authorization, including after upgrades.
Unix group/world-writable executable files are rejected. Windows requires native
`.exe` installations; `.bat`/`.cmd` shell wrappers are deliberately unsupported.
Help/version inspection has a five-second deadline and bounded output. Required
options are checked on each preview. Unknown or inadequate CLIs fail closed.
There is no signature verification; only select installations you trust.

Codex runs noninteractively with stdin task data, JSONL progress, `never`
approvals, `workspace-write`, ephemeral sessions and no user config/execpolicy
rules and the shared daemon disabled. Command sandbox network and temporary-directory writes are disabled.
Forma never requests `danger-full-access`, approval bypass or auto-review.
Provider authentication uses existing OS-user authentication files. The child
environment is an allowlist of OS paths, home/config locations, locale and PATH;
API keys, tokens, loader variables and arbitrary app environment are not copied.
No credentials are collected or stored by Forma.

Claude Code 2.1.300 runs `--restricted` print mode with stream-json/verbose
output, `dontAsk`, no session persistence, a fixed
Read/Edit/Write/Glob/Grep tool set, an empty strict MCP configuration and disabled
hooks. Restricted mode confines file tools to the working directory and ignores
user/project/local settings; shell tools are absent. These are provider-level
restrictions, **not an OS sandbox** of the CLI process. The warning and separate
explicit risk acknowledgement are mandatory. Forma does not offer unrestricted shell mode.
Provider configuration/authentication remains provider-owned; Forma makes no
claim to enforce OS-level isolation of the entire provider process.

## Approval and workspace lifecycle

The backend generates a single-use preview from saved issue identifier/title,
Markdown description (including acceptance criteria when present), project
name/context and repository/base commit. Data is JSON encoded and identified as
untrusted. Repository files, issue instructions and agent output cannot authorize
permissions. Prompts are not persisted in Forma's run table. M06 stores approved
queue snapshots until dispatch, cancellation, expiration or restart invalidates
them; these snapshots can appear in backups made while tasks await dispatch.

A preview expires in five minutes. Start revalidates the executable digest,
settings, issue timestamp, authorized repository identity and HEAD. Changed or
expired previews require fresh approval. The default process budget is one and can explicitly be configured to two,
shared with independent tests. A SQLite unique partial index prevents duplicate
active issue runs. Queue dispatch requires Start Next or Start Approved Queue.

A narrow libgit2 service creates a new UUID branch
`forma/agent-F<project>-<issue>-<UUID>` and a linked worktree under app-data
`agent-worktrees/<run UUID>`. Existing branches/paths are never overwritten.
The user's checked-out tree is untouched. Dirty, unborn, detached, bare,
operation-in-progress repositories, tracked symlinks and submodules are rejected.
Partial creation failures retain artifacts and record a failed run. The worktree
service never executes Git hooks or invokes a shell. Forma never commits, pushes or merges. Confirmed cleanup is available only for
known clean, reviewed worktrees.

Worktrees share Git metadata with their parent. They are a working-directory
isolation mechanism, **not a security boundary against a malicious provider**.
Provider processes have the current OS user's privileges; provider-enforced
command restrictions are narrower than whole-process isolation. Prompt injection
cannot be eliminated by instructions. Review the task and repository, trust the
selected executable, use a separate OS account/container when stronger isolation
is required, and review all changes before manual integration.

## Supervision, progress and results

States are Queued → Running → Completed/Failed/Cancelled. Spawn and worktree
failures are Failed; timeouts are Failed with a specific timeout result. UTC
start/end timestamps, exit code, result, bounded logs, branch/base/repository
references and review decision persist in append-only migration 005.

A Rust worker drains stdout/stderr through a bounded 32-message channel. Lines
are limited to 8 KiB; overlong lines are omitted. At most 64 records are drained
before cancellation/deadline checks. Raw output and retained logs have a configured
1024–1048576-byte limit (default 65536); exhaustion terminates the process with a
specific failure result. Snapshots update at most five times a
second; mounted panels poll once a second, retaining one 20-run page. Recent
JSON event types are rendered as text, alongside raw bounded text logs. No agent
HTML, Markdown, links or images execute or load remote content.

Common key/token/password patterns and private-key blocks are redacted before
Forma displays or persists output; diff text uses the same heuristics. Redaction
is not comprehensive: encoded, split or novel secrets may escape detection.
Backups/logs can contain private code or task content and are not encrypted.
Provider-owned logs/auth files follow the provider's own retention policies.

Cancellation, timeout, normal exit and Rust unwinding kill the process group on
Unix or the Windows Job Object, then reap the leader. Surviving ordinary children
are terminated after a successful leader exit too. A pipe watchdog kills the
Unix group if Forma exits/crashes; Windows jobs use kill-on-close. Processes that
intentionally escape the group/session or exploit provider permissions cannot be
contained by this mechanism. On restart, Queued/Running rows become Interrupted;
execution never resumes automatically. Worktrees are retained for inspection.
The desktop holds an exclusive app-data lock to prevent two live supervisors.

Result inspection verifies the retained worktree path, common Git directory and
branch before reading it. Diffs compare the recorded base to the index/workdir,
including untracked files, list up to 500 paths, limit text to 64 KiB and treat
files over 1 MiB as binary. Counts reflect the full libgit2 diff. Ignored files
and binary content require manual review. libgit2 still allocates repository
status/diff metadata; this is not an OS memory/disk quota. Test claims are
untrusted provider reports in logs, not an independent test certification.
Accept/reject never integrates or deletes changes.

## Backup and recovery

Format 4/schema 6 backs up settings, run metadata and bounded redacted logs, but
never credentials, repository contents or temporary worktrees. Exact released
M03 format 1/schema 3 and M04 format 2/schema 4 and M05 format 3/schema 5 exports validate first and migrate
in private staging. Imported repository/executable/worktree paths lose access
authorization; restored runs are historical and cannot open worktrees through
Forma. Select executables/repositories again before new runs. Old worktree paths
remain visible for deliberate manual inspection. Restore/deletion is blocked
while execution is active. See [BACKUP](BACKUP.md).

## Versions and verification

Codex CLI 0.162.1 is the installed version used for M05 verification. Adapters
require actual advertised flags rather than assuming all versions work.
Claude Code 2.1.300 was installed in a temporary directory for actual version/help
and restricted launch verification. It reported the expected five tools and no
MCP servers, then exited 1 because authentication was unavailable. Authenticated
Claude task completion remains unverified. Compiled simulated adapters exercise
both provider lifecycles separately from real provider execution.
See [M05](../M05.md) for actual commands/results and platform status.

Primary interface references:
[Codex noninteractive mode](https://learn.chatgpt.com/docs/non-interactive-mode),
[Claude programmatic execution](https://code.claude.com/docs/en/headless),
[Claude CLI reference](https://code.claude.com/docs/en/cli-reference), and
[command-group process groups/jobs](https://docs.rs/command-group/5.0.1/command_group/).

## M06 reliability controls

[MULTI-AGENT](MULTI-AGENT.md) defines the explicit queue session, per-issue
exclusivity, shared process budget, launch-time revalidation and worktree cleanup
rules. Independently approved project tests now capture their own exit code,
duration and bounded output; they never promote issues to Done. Restarts and
restores invalidate unused approvals rather than replaying execution. Provider and
test-executable authorizations never transfer through a backup restore.

Whole-provider OS containment is unavailable. Enabling **Require whole-provider
OS isolation** blocks execution and never falls back. Codex/Claude restrictions
and residual host exposure remain as described above. M06 real provider and native
platform evidence is recorded in [M06](../M06.md); M05 evidence is historical.

## M08 guided onboarding

Use **Set up AI** in Agent settings or **Run with AI** in a saved issue. The
[onboarding guide](AI-ONBOARDING.md) describes installation, authorization,
authentication, repository connection, permissions and first-task approval.
Discovery checks PATH and documented native locations without executing candidates.
Explicit authorization binds a five-minute scan token to its canonical path and
SHA-256, reuses capability inspection and saves the existing provider settings.
No arbitrary executable path IPC is exposed. Native picker authorization remains.

Only authorized, unchanged executables can run documented `codex login status`
or `claude auth status`. Browser login (`codex login` / `claude auth login`) needs
an explicit click, uses bounded process supervision and supports cancellation.
No credential files are inspected by Forma; login output is discarded and never
saved. Missing/unknown status cannot report readiness. Installation is manual and
provider-owned; official guides and copyable commands do not execute installers.

The recommended settings action is explicit and preserves any whole-provider OS
isolation requirement. It never grants per-task approval or Claude risk consent.
First-task setup opens the same existing exact-prompt preview and coordinator;
worktrees are created only after execution approval. Existing queue, progress,
cancellation, results and review remain the only task execution lifecycle.
