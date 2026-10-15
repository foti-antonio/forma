# Private beta tester guide — 0.1.0-beta.1

**Preparation only; enrollment has not started.** This opt-in program is limited to
5–10 external testers. There is no public release. Read this guide and the
[security model](SECURITY-MODEL.md) before agreeing to participate. Use a separate
test OS account with sample data and disposable repositories. You may decline
agent testing, stop at any time, or withdraw without giving a reason.

## Coordinator handoff

The owner manually chooses a coordinator, a private feedback channel and a test
window before contacting anyone. Those assignments are pending. Obtain explicit
opt-in before providing a package; record only tester aliases T01–T10, consent
date, assigned platform/package and checklist status in a private roster. Keep
contact details outside this repository. Do not auto-invite, publish a signup
form, or treat these documents as permission to distribute publicly.

Plan for Ubuntu 24.04 x86_64 (DEB and AppImage), Windows 10 and 11 x64 (NSIS and
MSI separately), and macOS 15+ on Intel and Apple Silicon. Five to ten people may
cover several packages; missing coverage remains a blocker. Record exact OS point
versions: a macOS 15-family CI result does not prove every macOS 15+ version.
Linux ARM, Windows ARM and universal macOS installers are unavailable.

Before each handoff, the coordinator must resolve applicability of the
[blockers](RELEASE-BLOCKERS.md), verify artifact availability and checksums, and
record candidate version, source SHA, file name/hash, recipient alias and date.
Use owner-approved access-controlled transfer for the cohort. The repository and
existing Actions artifacts may be public: **an Actions URL is not a private
access control**. Do not upload tester results or personal data there. Do not
redistribute the candidate or forward access. No transfer or invitation is
performed by this milestone.

## Candidate and safety

The reviewed M07.1 candidate was built from
`2ef872e73db6befcb84dc443b11d9a3224728e6a` in
[run 38819383958](https://github.com/foti-antonio/forma/actions/runs/38819383958).
The [six-installer archive](https://github.com/foti-antonio/forma/actions/runs/38819383958/artifacts/12012234042)
is time-limited (30-day retention); availability has not been reverified in M07.2.
The coordinator must supply the matching
[reviewed SHA-256 list](releases/m07.1-v0.1.0-beta.1.SHA256SUMS) through the agreed
channel. Do not use M07's older checksum list. If expired, stop and request a
reviewed replacement with a new source/hash record. Rebuilt or signed packages
have different hashes and require new acceptance records.

- Windows packages are unsigned; macOS packages are ad-hoc signed, without
  verified Developer ID/notarization. A matching SHA-256 proves byte identity,
  **not publisher trust**. If SmartScreen, Gatekeeper, antivirus or organizational
  policy blocks installation or launch, stop and report **Blocked**. Do not
  override warnings, remove quarantine, change security policy or run as root.
  Windows/macOS sessions may remain blocked until trusted builds are available.
- Backups and recovery copies are **unencrypted**, include all workspaces and
  possibly task/code/log content, and have no automatic expiration. Use only test
  data, protect local storage and deliberately remove obsolete test exports after
  verifying recovery. Never submit databases or backups as feedback.
- AI agents and independent tests execute as your OS user. Worktrees are **not
  OS security sandboxes**; code may access home files, credentials and network.
  Provider inference can leave the machine and incur provider charges. Default to
  no agent execution. Only a separately agreed agent session with a trusted
  executable and disposable local repository is appropriate. Do not enter or send
  provider credentials to Forma reports. Approval of a task does not authorize
  independent tests or manual Git integration.

## Verify, then install

Extract the entire supplied candidate ZIP first. On Linux, in that directory:

```sh
sha256sum -c SHA256SUMS
```

On macOS, in the extracted directory:

```sh
shasum -a 256 -c SHA256SUMS
```

All six entries must say OK when using the complete archive. Compare SHA256SUMS
itself against the reviewed list supplied by the coordinator, not just a sidecar
from an untrusted download. For a single installer, run `sha256sum <filename>` or
`shasum -a 256 <filename>` and compare the entire 64-character digest to that
reviewed list. Any mismatch or missing expected entry means stop.

On Windows PowerShell, set the file to your chosen NSIS EXE or MSI and compare
with its reviewed entry (these are the exact M07.1 bytes):

```powershell
$file = '.\Forma_0.1.0-beta.1_x86_64-pc-windows-msvc.exe'
$expected = 'd95d6ed9643f3182114c6fe2a3f64d955f553d1a0a8c8a2fb0c61380790a65d6'
# For MSI instead, use BOTH of the following assignments:
# $file = '.\Forma_0.1.0-beta.1_x86_64-pc-windows-msvc.msi'
# $expected = '3e46da3ac3492faf53c299177637c22a5dcf5639543f3d562f138398267f3ebb'
$actual = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
if ($actual -ine $expected) { throw 'Checksum mismatch: stop and report' }
$actual
```

Record file name and actual hash in your acceptance sheet before opening it.
These commands are instructions, not claims that verification ran on your OS.

| Platform                | Installation after successful verification                                                                                                                                                                                                                                                                                            |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ubuntu 24.04 x86_64 DEB | In the extracted folder: `sudo apt install ./Forma_0.1.0-beta.1_x86_64-unknown-linux-gnu.deb`, then launch `forma` as your ordinary test user. Requires a graphical session and distribution WebKitGTK 4.1.                                                                                                                           |
| Linux x86_64 AppImage   | `chmod +x Forma_0.1.0-beta.1_x86_64-unknown-linux-gnu.AppImage`, then `./Forma_0.1.0-beta.1_x86_64-unknown-linux-gnu.AppImage`. If FUSE is unavailable, use `./Forma_0.1.0-beta.1_x86_64-unknown-linux-gnu.AppImage --appimage-extract`, then `./squashfs-root/AppRun`. Record the launch mode. Other distributions need separate QA. |
| Windows 10/11 x64       | Open the verified EXE **or** MSI in Explorer, follow the wizard, record scope/destination and launch Forma from Start. Do not install both at once. NSIS defaults to per-user; MSI follows its wizard. Evergreen WebView2 is required; bootstrap may need internet. Stop at a security block.                                         |
| macOS 15+ Intel         | Open `Forma_0.1.0-beta.1_x86_64-apple-darwin.dmg` in Finder, drag Forma into Applications, eject, launch from Applications. Stop at a security block.                                                                                                                                                                                 |
| macOS 15+ Apple Silicon | Same steps with `Forma_0.1.0-beta.1_aarch64-apple-darwin.dmg`. Preserve download quarantine; do not use a trust override.                                                                                                                                                                                                             |

Use the [acceptance sheet](BETA-ACCEPTANCE.md) for first launch and all subsequent
tests. See [installation](INSTALLATION.md) for runtime requirements and data paths.
Do not use a real workspace as a test fixture. An empty first launch is expected
only in a new test account; an existing data directory is reused.

## Finish or stop

There is no updater. Export test data before an owner-assigned upgrade; install
the assigned newer compatible package over the old one and verify persistence.
Without two reviewed versions, record upgrade as Not run. Do not downgrade a
newer database. Restore only with a compatible build.

Close Forma before uninstalling. DEB: `sudo apt remove forma`; AppImage: delete
the image and any extracted `squashfs-root`; Windows: Settings → Apps → Forma →
Uninstall; macOS: remove Forma from Applications. App data remains separately at
Linux `${XDG_DATA_HOME:-~/.local/share}/dev.forma.desktop/` (default
`~/.local/share/dev.forma.desktop/`), Windows `%APPDATA%\dev.forma.desktop\`, or
macOS `~/Library/Application Support/dev.forma.desktop/`. Preserve worktree changes
and verify backups before deliberately removing test data through your OS file
manager. Uninstall does not erase exports, provider logs or credentials.

Send only the [bug form](BETA-REPORT.md), [feedback form](BETA-FEEDBACK.md) and
sanitized acceptance results through the coordinator's agreed private channel.
Check every excerpt and screenshot manually; automatic redaction is incomplete.
A public GitHub issue is optional only after sanitization and explicit consent.
For suspected vulnerabilities use [SECURITY.md](../SECURITY.md); pause the affected
scenario and preserve evidence locally without uploading sensitive files.
