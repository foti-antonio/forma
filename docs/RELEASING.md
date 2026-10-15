# Release procedure

Proposed version **v0.1.0-beta.1**, unpublished. Never create a release tag, publish
a release, merge to main or enable automatic distribution without final explicit
user approval. Branch pushes build review candidates only. There is no updater.

## Candidate preparation

Use README's Node/pnpm/Rust environment and Python 3.11+ for the artifact tooling.

1. Work on `milestone/m07.1-release-hardening`, starting from the verified M07 lineage.
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
   source SHA, manifest, artifact URLs and native QA evidence in M07.1.md.
   Archives expire after 30 days; rebuild from the reviewed source when necessary.
   Unsigned/ad-hoc CI artifacts are internal candidates only.

## Protected signing setup (currently blocked)

The repository owner must provision a GitHub environment named `release-signing`
with required reviewers, prevent self-review, restrict deployment refs to the one exact approved
version tag (currently `v0.1.0-beta.1`, type **Tag**, no branch policies), and restrict who can create matching tags using repository rulesets.
The release policy job reads the environment with `actions: read` and fails unless
reviewer protection, prevent-self-review and exact-tag-only deployment are present.
Disable administrator bypass in environment settings and verify this manually
before approval; the REST policy gate does not attest that setting. This read permission is
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

## Acquiring and configuring signing credentials

No credentials, environment, subscription or purchase is created by this work.
The owner must complete these steps on trusted machines, then review the exact
source before allowing any signing job. Never use a self-signed certificate to
satisfy public publisher trust.

### Windows owner steps

1. Choose a publicly trusted code-signing issuer/service and complete its identity
   validation. Confirm support for unattended GitHub-hosted signing before paying.
   [DigiCert's issuance requirements](https://knowledge.digicert.com/alerts/code-signing-changes-in-2023)
   require hardware-backed keys. A new certificate cannot be assumed exportable
   as PFX. A USB token normally cannot be attached to an ephemeral hosted runner.
2. The checked-in workflow supports **only an issuer-permitted exportable PFX**.
   If that is unavailable, signing is blocked until a reviewed hardware/cloud
   integration replaces certificate import with Tauri's documented
   [custom signCommand](https://v2.tauri.app/distribute/sign/windows/).
   Configure that service's least-privilege identity and timestamping, then keep
   the existing executable/installer signature gates. Do not export an HSM key.
3. For an existing permitted PFX, export the code-signing certificate and private
   key with its certificate chain and a strong password. In GitHub repository
   Settings → Environments → `release-signing` → Environment secrets, set
   `WINDOWS_CERTIFICATE` to the single-line base64 of the PFX and
   `WINDOWS_CERTIFICATE_PASSWORD` to that password. Use a local private file and
   GitHub's secret input; do not paste either into issues, logs or this repository.
4. After a separately approved tag run, require SHA-256/RFC3161 signatures,
   the expected certificate thumbprint, timestamp certificate and `Valid` status
   for `forma.exe`, NSIS and MSI. Test the downloaded packages under ordinary
   SmartScreen policy; certificate validity alone does not prove reputation.

### Apple owner steps

1. Enroll/maintain an Apple Developer Program team with permission to issue
   [Developer ID certificates](https://developer.apple.com/help/account/certificates/create-developer-id-certificates/).
   On a trusted Mac use Xcode → Settings → Accounts → team → Manage Certificates
   to create **Developer ID Application** (not Apple Development or Installer).
   Alternatively generate a CSR in Keychain Access and request that certificate
   in Certificates, Identifiers & Profiles, then install the returned certificate.
2. In Keychain Access → My Certificates, locate that identity with its private key,
   export both as password-protected `.p12`, and retain a secure offline copy.
   `security find-identity -v -p codesigning` shows the full identity; it must begin
   `Developer ID Application:`. The team ID is in developer account membership.
3. Set environment secrets `APPLE_CERTIFICATE` to the P12's single-line base64,
   `APPLE_CERTIFICATE_PASSWORD` to its export password,
   `APPLE_SIGNING_IDENTITY` to the complete identity, and `APPLE_TEAM_ID` to the
   membership team ID. Set `APPLE_ID` to the account email. Generate an
   [app-specific password](https://support.apple.com/en-us/102654) in the Apple
   Account sign-in/security page and store it as `APPLE_PASSWORD`; never use the
   primary account password. Two-factor authentication must remain enabled.
4. After an approved protected run, verify `codesign --verify --deep --strict`,
   `spctl --assess --type execute`, accepted app/DMG notarization and stapled
   tickets on both architectures. Download through a normal browser and test
   Gatekeeper with quarantine intact on clean Macs. Never strip quarantine.

### GitHub owner steps

In repository Settings → Environments create `release-signing`, add an independent
required reviewer with read access, enable **Prevent self-review**, disallow
administrator bypass, and select **Selected branches and tags**. Add only a
**Tag** rule equal to the approved version, with no wildcard or branch rules.
An owner alone cannot self-approve; arrange an eligible independent reviewer.
If the current plan cannot enforce these controls, change plan/visibility through
an explicit owner decision before proceeding. The environment REST gate checks
reviewer and exact tag rules; the reviewer must also inspect the UI's bypass policy.

In Settings → Rules → Rulesets create an active tag ruleset for version tags,
restrict creation/update/deletion to authorized release maintainers, and review
bypass actors. Record the ruleset and approval evidence before creating any tag.
Current audit found no environment, repository secrets or rulesets. This milestone
does not create them. Ordinary branch CI must continue without signing secrets.

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
