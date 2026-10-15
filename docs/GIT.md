# Local Git integration (M04)

This document describes repository inspection and issue links. M05 separately
adds approved branch/worktree creation for coding agents; see
[AGENTS-INTEGRATION](AGENTS-INTEGRATION.md) for that execution boundary.

Expand **Git repository** in a project and choose **Connect repository…**. Select
an existing repository root with the native directory picker. Forma accepts
normal repositories, linked worktrees, `.git` directories and bare repositories;
it does not discover a repository in a selected folder’s parents. Folder
selection is cancellable. Validation must succeed before a connection is saved.

Each project has **one active repository**. Reconnecting a different folder
atomically disconnects the previous one; its issue links remain. Multiple
projects may inspect the same repository. **Disconnect** requires confirmation
and changes only Forma’s SQLite database. Reconnecting the same canonical path
reactivates the same historical repository record. Moving a repository to a
new path creates a new record; existing links retain their original provenance.
Deleting a project/workspace cascades through its connections and links.

## Inspection

The panel displays name, canonical path, current branch (or detached/unborn HEAD),
working-tree counts, local branches and commits reachable from HEAD. Commit rows
show author name, UTC timestamp, abbreviated hash (full hash on hover), and a
bounded message. Bare repositories display history and branches without status.
Repositories with no commits have an explicit empty state. Missing, inaccessible,
invalid-index and damaged-object failures show an error with refresh/reconnect.
A successful read reports “Repository readable”; this is not a full `git fsck`.

Status includes staged and unstaged changes. Added counts index additions;
untracked counts individual files recursively, excluding ignored files. Modified
includes content/type changes; deleted includes either index or worktree deletion.
A file may contribute to multiple categories (for example staged addition then
worktree modification). Conflicts have a separate count. Rename detection is off;
renames normally appear as deletion plus addition. Submodule contents are not
recursively inspected. Status is a full local scan, not a paginated diff viewer.

Opening the panel, changing branch search/page, and **Refresh Git** request reads.
There is no timer, watcher or background polling. UI pages retain 20 commits and
25 branches. Rust accepts at most 100 per request and offsets through 10,000.
Branches are enumerated in libgit2 reference order; search is a literal,
case-insensitive substring. Enumeration stops after 10,000 branches and reports
truncation. Refine or use external tools for repositories beyond that ceiling.
History walks HEAD ancestry in libgit2’s default reverse chronological traversal,
without requesting a full-history time sort. Other branches’ commits can be
linked by their full object hash even when absent from HEAD history.

## Issue associations

Expand **Git activity** in a saved issue. Enter an exact local branch name or a
full 40-character SHA-1 commit hash, choose **Review Git link**, and confirm the
repository/target/issue shown. Multiple branches and commits may be linked.
Saving validates the issue’s project, repository ownership, active authorization,
target type and existence in Rust. Each issue supports up to 200 links. Duplicate
links are idempotent. **Remove Git link** also requires confirmation.

Suggestions inspect the first 25 branches and first 20 HEAD commits and match
the issue’s exact case-sensitive identifier, for example `F12-7`. Non-alphanumeric
boundaries allow `feature/F12-7-work` and `(F12-7)` while rejecting `F12-70`,
`xF12-7`, `F12-7x` and other project keys. Suggestions never save themselves;
clicking opens the same explicit confirmation. These are bounded hints, not an
exhaustive repository search. Branch names and messages remain plain inert text.

Each link stores its branch name or commit hash and commit metadata **at linking
time**. A renamed/deleted branch keeps its historical name, hash, author, message
and timestamp. Links never silently retarget; remove and recreate a link when
needed. History remains readable when a repository is disconnected, missing or
corrupt. Git activity does not change issue descriptions or Kanban state.

## Security and persistence

UI → feature store → typed IPC → Rust command → Git service → SQLite repository
and local libgit2 reader. Commands use `spawn_blocking`; scans never execute on
the WebView event thread. The database mutex serializes operations, including
Git reads; an unusually slow scan may delay other database work. UI requests use
generations/unmount guards to discard obsolete results. There is no subprocess
execution, shell plugin, filesystem plugin, remote client or new capability.

`git2` 0.21 uses libgit2 with default SSH/HTTPS features disabled. Production code
only opens repositories, resolves local references/objects and reads status and
history. It never runs Git hooks, credential helpers, Git config commands,
external diff/clean/smudge processes or repository scripts. There is no push,
pull, fetch, reset, checkout, commit, index write or repository mutation endpoint.
Status explicitly disables index stat-cache updates and rename detection.
Libgit2 necessarily interprets repository metadata (including worktree gitlinks,
object alternates and working-directory configuration) for local reads; selecting
a repository authorizes these associated local reads, not arbitrary WebView path
access. No network operation is invoked, including for partial-clone objects.

React cannot supply repository paths: only Rust receives native-picker results.
UUIDs and associations are scoped in services, SQL is parameterized, and SQLite
foreign keys plus a trigger enforce repository/issue project ownership. Git
errors are generic and never log libgit2’s potentially sensitive error/config
strings. Metadata is bounded and rendered as text; no author email/config values
are shown. Non-Unicode paths are rejected; invalid message text is replaced
lossily and control characters removed. Branch discovery reports invalid UTF-8
names as a readable error rather than passing an unusable name back to commands.

Migration 004 is append-only; prior migrations are unchanged. Connections,
historical links and authorization survive ordinary restarts. Backup format 2
includes connections and saved metadata **but not repository content**. On restore,
all imported paths lose authorization, regardless of their stored flags. Forma
can display them but cannot open them until native directory selection validates
the folder again. Missing paths therefore never break restore. M03 format 1 /
schema 3 exports are validated against their exact released schema, copied into
private staging and upgraded before confirmation/replacement. See [BACKUP](BACKUP.md).

## Limits and future work

This milestone is read-only. SHA-256 repositories, remote branches, diff content,
commit creation, branch renaming, automatic issue transitions and network Git
are outside M04. Large status scans allocate libgit2’s full status list, and
cancellation is not yet implemented. Measurements and platform verification are
recorded in [M04](../M04.md). Separate workers/cancellation and native Windows/macOS
interactive testing are recommended before adding more repository operations.

## M06 concurrent worktree lifecycle

Agent execution creates one UUID branch/worktree per run through libgit2. The
service serializes creation even for concurrent tasks in the same repository.
The primary working tree is never the execution directory. Worktree Management
allows explicit opening and confirmed cleanup of reviewed, clean, known linked
worktrees; branches and historical run metadata remain. Unknown, modified, ignored,
locked, symlinked and unreviewed contents block cleanup. Worktrees share Git
metadata and are not security sandboxes. See [MULTI-AGENT](MULTI-AGENT.md).
