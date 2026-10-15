# Installing the proposed beta

Version: **0.1.0-beta.1**. This is an unpublished candidate. There is no approved
public download yet. Candidate artifacts are attached to the Actions run linked
in [M07](../M07.md), retained for 30 days and may require GitHub authentication.
Choose `forma-beta-candidate` for all six installers and `SHA256SUMS`, or the
matching target archive. Extract before verifying or installing.

| System        | Target in filename       | Requirements                                                       |
| ------------- | ------------------------ | ------------------------------------------------------------------ |
| Linux x86_64  | x86_64-unknown-linux-gnu | Graphical desktop; WebKitGTK 4.1; CI baseline Ubuntu 24.04         |
| Windows x64   | x86_64-pc-windows-msvc   | Windows 10/11; WebView2 Runtime                                    |
| Intel macOS   | x86_64-apple-darwin      | Configured minimum macOS 10.15; only hosted macOS 15 builds tested |
| Apple Silicon | aarch64-apple-darwin     | Native Apple Silicon macOS; hosted macOS 15 builds tested          |

Configured minimum versions are not claims of testing on every supported OS.
The frontend uses Tailwind 4, whose [browser baseline](https://tailwindcss.com/docs/compatibility)
includes Safari 16.4 / Chrome 111 / Firefox 128. The inherited macOS 10.15 native
packaging floor does not establish compatible WKWebView rendering. Selecting and
testing the actual minimum macOS/WebKit baseline is a public distribution gate;
only macOS 15 hosted initialization is currently evidenced.
Linux ARM, Windows ARM and universal macOS packages are not provided.

## Check the download

From the extracted candidate directory on Linux: `sha256sum -c SHA256SUMS`.
On macOS: `shasum -a 256 -c SHA256SUMS`. Both require the six listed packages;
for a single-target download use its `<target>.sha256` file instead.
On Windows compare `(Get-FileHash .\Forma_0.1.0-beta.1_x86_64-pc-windows-msvc.exe -Algorithm SHA256).Hash`
to its entry in `SHA256SUMS`. A checksum detects corruption, not publisher trust.

## Linux

For DEB on a supported Debian/Ubuntu installation:

```sh
sudo apt install ./Forma_0.1.0-beta.1_x86_64-unknown-linux-gnu.deb
forma
```

The DEB installs the executable under `/usr/bin`, its desktop entry under
`/usr/share/applications`, and icons under `/usr/share/icons/hicolor`.
Remove with `sudo apt remove forma`; app data is retained. The local Arch host
cannot establish a Debian installation result merely by extracting the package.

For AppImage, place it in a user-owned directory, make it executable and launch:

```sh
chmod +x Forma_0.1.0-beta.1_x86_64-unknown-linux-gnu.AppImage
./Forma_0.1.0-beta.1_x86_64-unknown-linux-gnu.AppImage
```

FUSE may be needed. The AppImage runtime also supports `--appimage-extract`;
launch `squashfs-root/AppRun` from the extracted directory. This is a FUSE-free
execution path, not proof of compatibility across distributions. AppImage removal
means deleting the image/extracted directory; persistent data remains separate.
Do not run Forma as root. WebKitGTK/system graphics compatibility still matters.

## Windows

Choose either NSIS EXE or MSI; avoid installing both simultaneously. Open the
installer and follow its installation wizard. NSIS uses a per-user installation
by default (normally `%LOCALAPPDATA%\Forma`); MSI follows the wizard's installation
scope/location. Confirm the destination in the wizard rather than assuming it.
WebView2 bootstrap may need internet access if the runtime is missing. Launch
Forma from Start, create sample data, close and reopen before trusting the install.
Remove through Settings → Apps; preserve app data for reinstall/backup.

Current unsigned candidates do not establish publisher identity. Signing and
clean-machine SmartScreen/enterprise-policy checks remain required for public
release. Do not disable security policy as a workaround; report the warning.

## macOS

Select the DMG for your CPU. Mount it in Finder, drag Forma into Applications,
eject the image, then launch Forma from Applications. Close/reopen with sample
data to verify persistence. Remove the application from Applications to uninstall;
app data remains separate.

Ad-hoc candidate signing is not Developer ID trust or notarization. Public
installation is blocked pending signed/notarized builds and clean-machine
Gatekeeper tests. Do not strip quarantine, disable Gatekeeper or recommend
security bypasses. A DMG mount/copy/launch on a CI runner is not that test.

## Data, updates and removal

Export a [backup](BACKUP.md) before replacing a build. There is no auto-update.
Install an approved newer package over the prior app only after reading migration
notes; downgrades may reject newer schemas. Restore with a compatible build.

Data lives under `dev.forma.desktop`: Linux `$XDG_DATA_HOME` (default
`~/.local/share`), Windows `%APPDATA%`, macOS `~/Library/Application Support`.
Deleting the application does not intentionally delete this directory. It contains
SQLite, recovery files and retained agent worktrees. Export backups and preserve
worktree changes before deliberately deleting data. Backups exclude worktree and
repository contents. See [troubleshooting](TROUBLESHOOTING.md).
