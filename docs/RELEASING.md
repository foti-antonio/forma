# Release procedure

Proposed version **v0.1.0-beta.1**, unpublished. Never create a release tag, publish
a release, merge to main or enable automatic distribution without final explicit
user approval. Branch pushes build review candidates only. There is no updater.

## Candidate preparation

Use README's Node/pnpm/Rust environment and Python 3.11+ for the artifact tooling.

1. Work on `milestone/m07-public-beta`, starting from the verified M06.1 lineage.
   Preserve `dev.forma.desktop`, all released migrations and user data.
2. Align `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and
   the Forma entry in Cargo.lock. Add CHANGELOG and `docs/releases/v<version>.md`.
   Run `node scripts/release-version.mjs` (also accepts `RELEASE_TAG`).
3. Native numeric mapping: beta N uses MSI `major.minor.patch.N` and Apple build
   `major.minor.(patch*10000+N)`. Stable reserves N=9999. Review native bounds and
   monotonic upgrade behavior for every release. These build versions are not the
   displayed SemVer. The beta.1 MSI is `0.1.0.1`, Apple build is `0.1.1`.
   Earlier internal `0.1.0` builds were not public releases; export before upgrade.
4. Run README's full quality checks. Push the milestone branch to run CI.
   Each desktop target checks frontend, Rust/IPC and Clippy before native packaging.
   The aggregate candidate job rejects missing packages, wrong headers/CPU,
   mixed source commits/versions, corrupt bytes and extra installers.
5. Download `forma-beta-candidate`, verify the six checksums, record the exact run,
   source SHA, manifest, artifact URLs and native QA evidence in M07.md.
   Archives expire after 30 days; rebuild from the reviewed source when necessary.
   Unsigned/ad-hoc CI artifacts are internal candidates only.

## Protected signing setup (currently blocked)

The repository owner must provision a GitHub environment named `release-signing`
with required reviewers, prevent self-review, restrict deployment refs to approved
version tags, and restrict who can create matching tags using repository rulesets.
The release policy job reads the environment with `actions: read` and fails unless
reviewer protection and prevent-self-review are present. This read permission is
required by [GitHub's environment API](https://docs.github.com/en/rest/deployments/environments#get-an-environment). No environment or credential is created by
this milestone. Availability of environment protections depends on repository
visibility/plan. Resolve that before any approved tag.

Environment secrets (never repository files or pull request logs):

- Windows: `WINDOWS_CERTIFICATE` (base64 PFX with private key),
  `WINDOWS_CERTIFICATE_PASSWORD`. The approved Windows job imports into the
  ephemeral runner's current-user certificate store and supplies a temporary Tauri
  certificate thumbprint configuration. Tauri signs with SHA-256 and an RFC3161
  timestamp endpoint. Both installers must pass `Get-AuthenticodeSignature` with
  `Valid`. Hardware-backed certificates/services need an explicitly reviewed
  replacement signing command; do not export a non-exportable key or weaken trust.
- Apple: `APPLE_CERTIFICATE` (base64 Developer ID Application P12),
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` (full Developer ID
  Application identity), `APPLE_ID`, `APPLE_PASSWORD` (app-specific password),
  `APPLE_TEAM_ID`. Tauri imports/signs/notarizes the app. The protected step
  additionally submits the signed DMG with `notarytool`, requires `Accepted`,
  staples its ticket and validates it. The workflow checks `codesign`, `spctl`,
  and app/DMG tickets; missing credentials or failed trust stops it. The pinned
  [CLI 2.12.2 signing source](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.2/crates/tauri-bundler/src/bundle/macos/sign.rs)
  handles certificate import; its [DMG bundler](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.2/crates/tauri-bundler/src/bundle/macos/dmg/mod.rs)
  signs the image, so image notarization/stapling is an explicit additional step.

Credentials exist only in signing steps on fresh hosted runners. No shell tracing,
secret printing, certificate upload or artifacts containing signing material.
Windows PFX is deleted in `finally`; temporary runner stores/config disappear
with the hosted VM. macOS import/cleanup is managed by Tauri's signing implementation.
Do not pass signing secrets to ordinary CI, PR builds or unreviewed code.

References: [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/),
[Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/).
Configured integration is not evidence of an executed signing/notarization run.
SmartScreen reputation and Gatekeeper on downloaded/quarantined artifacts need
separate clean-machine evidence; never bypass them to obtain a passing result.

## Final approval and draft

After required blockers in M07 are resolved, obtain explicit approval of the exact
source SHA and version before creating/pushing `v0.1.0-beta.1`. The tag workflow:

1. Validates the exact tag against all version metadata and release notes.
2. Verifies protected environment policy, then waits for reviewers before secrets.
3. Runs mandatory CI and rebuilds signed/notarized packages on all four targets.
4. Verifies six installers/checksums and creates a **draft prerelease** with the
   prepared release notes. It cannot publish a public release.

Verify the signed rebuild's new hashes and native trust/install behavior. Obtain
separate explicit final approval before manually publishing the reviewed draft
or merging to main. Do not infer approval from a successful CI run or a configured
environment. A draft is still unpublished; tag authorization is not publication
authorization. If any package is missing, do not distribute a partial set as the
complete release. If a workflow fails, retain logs and fix on the branch; do not
move an already distributed tag. Signed packages differ from unsigned candidates.

## Native acceptance record

Use isolated temporary OS data/accounts, trusted disposable Git repositories and
the compiled simulated agent fixture; never real coding agents on user repos.
Record installer scope/path, CPU, OS version, source SHA, package hash, first
launch, restart persistence and uninstall/data retention for each package.

On each architecture verify workspace/project CRUD, issue edit, keyboard Kanban,
Git folder/link flow, executable authorization, approval/cancellation, queue start
and stop, independent tests, worktree review/clean cleanup, export/restore and
restart. Check light/dark/system, 760×540 window scrolling, keyboard focus and
loading/error/empty states. Screenshots must be genuine app captures with sample
data. Record unsupported physical/GUI access as unverified, not passed.
