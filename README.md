# Forma

A local-first desktop project manager for developers. Fast, quiet, and yours.

M03 adds a keyboard-accessible Kanban board, persistent ordering, combined issue
filters, and native local backup/restore to the workspace, project and issue
workflows from M01/M02. SQLite stores your data and preferences. No account or
server is required; authentication, cloud sync and AI agents remain out of scope.

## Develop

Install Git, **Node.js 24 LTS**, **pnpm 11.3.0**, and Rust 1.90 or newer (current stable recommended) through
[rustup](https://rustup.rs). Enable pnpm through Corepack if available, or install
it with `npm install -g pnpm@11.3.0`.

### Linux

On Ubuntu 24.04 / Debian with WebKitGTK 4.1 packages:

```sh
sudo apt-get update
sudo apt-get install build-essential curl file libssl-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf xdg-utils
```

On Arch Linux: `sudo pacman -S --needed base-devel webkit2gtk-4.1 openssl curl wget file librsvg patchelf`.

Use a graphical desktop session. AppImage execution may additionally require
FUSE (or extraction with `--appimage-extract`). See the upstream
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for other distros.

### Windows

Install Visual Studio Build Tools 2022 with **Desktop development with C++**
and the Windows SDK. Install Microsoft Edge WebView2 Runtime if it is missing.
Use Rust's MSVC toolchain and run the commands below in PowerShell.

### macOS (Intel and Apple Silicon)

Install Xcode Command Line Tools: `xcode-select --install`. Install Rust for your
native architecture. Native builds produce the corresponding Intel or Apple
Silicon application; CI builds both architectures separately.

### Start

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

Create a workspace, then a project. Click a project name in a card or sidebar
for its issue list. The pencil or **Project settings** edits the project.
Choose **List** or **Board**; Forma remembers each project’s view.
Use **New issue** (or `N`) to add work, `/` to search, and status/priority/sort
controls to organize it. Both views combine text, status, priority, label and
inclusive due-date filters; **No due date** selects undated work. **Clear all
filters** resets them. Lists paginate by 25; boards retain at most 25 matching
cards per column, with independent Previous/Next controls.

On a board card, **Enter** opens details. **Alt + ↑/↓** reorders relative to
the next visible card, preserving hidden cards’ relative order. **Alt + ←/→**
appends to an adjacent status column. Drag a card’s handle with the pointer,
or use **Space**, arrows, **Space** to pick up/move/drop (**Escape** cancels).
Empty column bodies accept drops. Board ordering is always manual; list sorting
remains independent. Movement results are announced and failures refresh SQLite’s
canonical state. Click an issue to edit; `Ctrl/⌘ + Enter` saves. The editor
supports Markdown preview, labels, due dates, manual order, parent selection,
subtask creation, and paginated activity. **Labels** manages workspace labels.

Archive or restore an issue inside its editor, then use **Archived** to find it.
Archiving a parent leaves its children available. Deleting an issue preserves its
children as top-level issues. Unsaved edits require confirmation before discard;
failed saves retain form data. Project archive actions stay on project cards;
workspace rename/deletion remains at the bottom of the sidebar. Deletions require
confirmation. See [issue rules](docs/ISSUES.md) for numbering and hierarchy.

`pnpm dev` starts only Vite for frontend development. It intentionally displays a
desktop-required error in a regular browser; it does not simulate persistence.

## Quality checks

```sh
pnpm format:check
pnpm lint
pnpm typecheck
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path src-tauri/Cargo.toml --features ipc-tests
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Rust service/database tests can run without desktop system libraries:

```sh
cargo test --locked --manifest-path src-tauri/Cargo.toml --no-default-features
```

The optional IPC test uses Tauri’s test transport and the production command
handler against real SQLite. On a Linux Hyprland desktop, `python3
scripts/native-smoke.py` exercises real native forms and restart persistence.
Build the debug binary first with `pnpm tauri build --debug --no-bundle` and
install `wtype`, `grim`, and `tesseract`. Leave the test window focused; the script
aborts before sending keys to another app. It uses temporary app-data and prints
the artifact directory. This desktop-specific smoke is opt-in, not a headless CI
test.

Vitest disables Node's experimental global Web Storage so jsdom provides the
browser storage implementation, including on Node 26.

## Build and release

```sh
pnpm tauri build
```

Bundles appear under `src-tauri/target/release/bundle/`. CI requests Linux
AppImage/deb, Windows NSIS exe/MSI, and macOS app/DMG. Ordinary CI requires no
signing credentials. A matching `v0.1.0` tag runs the checks and builds, then
prepares a **draft** GitHub release with installers. Publishing remains manual.

macOS CI uses ad-hoc signing; this does not establish Developer ID trust or
notarization. Public distribution needs an Apple Developer ID certificate and
notarization credentials configured according to
[Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/).
Windows packages are unsigned and may trigger SmartScreen or enterprise policy.
Trusted releases need an Authenticode certificate/signing service; see
[Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/).
No updater is configured.

## Local data

The Rust backend creates `forma.sqlite3` in Tauri's app-data directory:

- Linux: `$XDG_DATA_HOME/dev.forma.desktop/` (normally `~/.local/share/dev.forma.desktop/`)
- Windows: `%APPDATA%\dev.forma.desktop\`
- macOS: `~/Library/Application Support/dev.forma.desktop/`

The active workspace, per-project List/Board selection and appearance are stored
in SQLite. M03 imports the previous local appearance preference on first launch;
WebView storage remains only an appearance cache.

Use **Export backup** in the sidebar to choose a destination with a native dialog.
**Restore backup…** validates a selected snapshot, asks for explicit replacement
confirmation, saves a recovery copy beside the live database, then restores and
refreshes the app. Local backups are **not encrypted**. Read [BACKUP](docs/BACKUP.md)
for format, compatibility, rollback and recovery instructions. Avoid copying an
open WAL database; use the built-in consistent snapshot export. Do not place real databases in the
repository; they are ignored by Git. Deleting a workspace cascades through its
projects, issues, labels and activity.

Read [ARCHITECTURE](docs/ARCHITECTURE.md), [ROADMAP](docs/ROADMAP.md),
[CONTRIBUTING](CONTRIBUTING.md), [SECURITY](SECURITY.md), and the actual validation
record in [M01](M01.md) and [M02](M02.md) and [M03](M03.md).

Licensed under [Apache-2.0](LICENSE).
