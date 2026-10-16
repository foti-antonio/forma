# AI setup and your first task

Open **AI agents & runs → Agent settings → Set up AI**, or choose **Run with AI**
in a saved issue. Save pending issue edits first. Choose Codex or Claude Code;
using a provider can require its account, paid usage and internet connection.
Forma never asks you to paste credentials.

## Guided flow

1. **Find installation.** Forma checks absolute PATH directories and known native
   installation locations, resolves canonical files and hashes them without running
   discovered software. Discovery alone means neither authorization nor readiness.
2. **Install and authorize.** If missing, open the official guide for your OS,
   install the CLI yourself, then rescan. Windows needs a native `.exe`; `.cmd`,
   `.bat` and PowerShell launchers are not supported. On macOS the wizard also
   checks both Homebrew prefixes. Each candidate shows its canonical path and
   SHA-256. Authorize only a provider you trust; this permits bounded version/help
   and authentication status checks. A native picker remains available for custom
   installations. Upgrades or identity changes require fresh authorization.
3. **Sign in.** Forma reports authenticated, unauthenticated, unknown or unavailable
   from the documented CLI status interface. Click **Open Codex login** or **Open
   Claude Code login** to start that provider's official browser flow. Finish in
   the provider browser, then **Check authentication**. Login has a three-minute
   deadline and explicit cancellation; closing setup cancels an active login.
   If the browser cannot open or the provider requires terminal input, run the
   displayed official login command in a terminal and recheck. Forma does not
   expose login output, device codes or interactive credential input.
4. **Connect your project.** The issue flow checks the current project's repository
   authorization. Select its existing Git folder with the native picker if absent
   or restored. Folder access permits inspection and approved task working-copy
   preparation. Use a clean repository with a committed branch; preserve local
   changes yourself. No Git knowledge about creating worktrees is needed.
5. **Review permissions.** The optional recommended configuration uses one process,
   600 seconds, 65,536 log bytes and the provider's existing conservative policy.
   Applying it is explicit and preserves an existing whole-provider isolation
   requirement. That requirement blocks runs because OS containment is unavailable.
   Acknowledging the explanation never approves execution or Claude host risk.
6. **Preview first task.** Read the exact saved-issue prompt, executable/version,
   repository/base and permissions in the existing approval dialog. Choose
   **Approve & run once** to authorize one execution. Claude additionally requires
   its separate host-risk consent. The existing queue alternative remains available.
   Nothing runs automatically at setup completion, startup or restore.

Settings-only setup ends by saving the preferred provider. Repository connection
happens in the saved issue flow. Setup progress is recomputed from Rust-owned
settings, filesystem identity, provider status and repository authorization; there
is no new browser persistence or stored credential/onboarding database.

## Installation references and detection

Instructions were checked against official documentation on 2026-10-16:

- [Codex CLI installation](https://developers.openai.com/codex/cli/).
- [Codex installer locations](https://learn.chatgpt.com/docs/config-file/environment-variables):
  `~/.local/bin` on Linux/macOS and `%LOCALAPPDATA%\Programs\OpenAI\Codex\bin`
  on Windows; an absolute `CODEX_INSTALL_DIR` is also discoverable.
- [Claude quickstart](https://code.claude.com/docs/en/quickstart) and
  [native installation locations](https://code.claude.com/docs/en/setup):
  `~/.local/bin/claude` / `%USERPROFILE%\.local\bin\claude.exe`.
- [Codex login/status](https://learn.chatgpt.com/docs/developer-commands?surface=cli)
  and [Claude auth login/status](https://code.claude.com/docs/en/cli-reference).

Windows discovery also checks the documented per-user
[WinGet portable links directory](https://github.com/microsoft/winget-cli/blob/master/doc/specs/%23182%20-%20Support%20for%20installation%20of%20portable%20standalone%20apps.md),
`%LOCALAPPDATA%\Microsoft\WinGet\Links`, so a supported portable installation
can be found before Forma receives a refreshed PATH. No package-manager query is
executed during discovery.

The UI offers the documented `brew install --cask claude-code` on macOS and
`winget install Anthropic.ClaudeCode` on Windows as selectable, copyable text.
These need the corresponding package manager installed. Linux instructions link
to the provider guide rather than embedding a downloaded-script command. Forma
never invokes installers, downloads scripts, changes PATH or modifies system
configuration. Git/Node/npm prerequisite checks inspect file presence on PATH,
without executing them or claiming version compatibility; native provider installs
need no Node/npm. Forma uses bundled libgit2 for repository operations.

Detection is bounded to 128 absolute search directories and 16 unique executable
candidates per provider. Unsafe/inaccessible files are skipped. A scan produces
five-minute in-memory tokens bound to canonical path, provider and digest;
rescan invalidates that provider's old tokens. Authorization accepts a token,
never a renderer-supplied path. Explicit confirmation, fresh identity checks and
existing capability verification precede saved authorization. New/unknown CLI
interfaces fail closed; a provider update may therefore need a compatible version
rather than merely the newest version.

## Progress and recovery

The existing run panel shows progress, bounded inert logs, cancellation and result
inspection. Review the diff and any independently approved tests, then accept or
reject for manual integration. Forma creates a separate branch/working copy only
following execution approval and never commits, merges or pushes.

Actionable errors explain installation, incompatible security flags, missing
sign-in, changed executable identity, disconnected/dirty repositories, permission
restrictions, timeouts and failed runs. **Technical diagnostics** and bounded logs
retain details as plain text. Provider output remains untrusted.

Authentication status is a point-in-time CLI report, not proof of available
credits, network reachability or future task success. Status commands run only
for an authorized unchanged executable and use the existing sanitized environment.
Codex needs recognized logged-in output plus exit 0; Claude needs valid JSON with
`loggedIn: true` plus exit 0. Unknown output and timeouts never imply readiness.
The CLI, not Forma, accesses provider-owned authentication. Forma reads no private
credential files and persists no login output, URL, token or account identifier.

**Git worktrees are not OS security sandboxes.** Providers run as your OS user and
may access host files, credentials and network. Existing Codex restrictions,
Claude reduced tools/separate consent, SHA-256 revalidation, single-use task
approval, process supervision and repository authorization remain in force.
Read [the security model](SECURITY-MODEL.md) and
[agent integration](AGENTS-INTEGRATION.md). Native/platform verification and
remaining friction are recorded in [M08](../M08.md).
