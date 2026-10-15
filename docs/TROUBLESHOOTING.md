# Troubleshooting

Include OS/architecture, Forma version, package filename/SHA-256, reproduction
steps and the exact sanitized error. Distinguish a process that stays alive from
a window that renders and responds. Never post credentials, private backups,
repository paths/content or unreviewed provider logs publicly.

## Forma does not open

Confirm the CPU matches the target and verify the download checksum. On Linux,
start from a terminal to capture a sanitized error; verify WebKitGTK 4.1 and
system graphics dependencies. An AppImage may need FUSE; use its supported
`--appimage-extract` path to diagnose FUSE separately. AppImages are not guaranteed
to run on every distribution or on older glibc than the build baseline.
On Windows confirm WebView2 and installation scope. Installer/runtime downloads
may require network access. On macOS retain the exact Gatekeeper message and
confirm Developer ID/notarization status. Do not disable security controls,
remove quarantine or use ad-hoc signing as a trust workaround.

Only one Forma process may own an app-data directory. Close the prior process
normally; do not delete data/locks while it is running. If startup reports a
newer unsupported schema, use a compatible newer build; do not edit schema
versions manually. Preserve data before reinstalling.

## Data appears missing

Check the OS account and app-data location in [INSTALLATION](INSTALLATION.md).
Development/smoke tests intentionally use isolated temporary data. The UI may be
showing a different workspace, archived records or active filters. Clear filters
and inspect workspace selection before restoring. Uninstalling the app and
deleting its data directory are separate operations.

## Backup or restore fails

See [BACKUP](BACKUP.md). Use Export backup for a consistent open-database snapshot;
do not copy an active WAL database alone. Wrong versions, corrupt files, extra
schema objects and foreign-key failures are rejected without replacing live data.
Active execution blocks restore. Confirm replacement deliberately. Keep the
recovery snapshot and original data until the restored state has been inspected.
Backups exclude repositories/worktrees and are not encrypted.

## Agent or test cannot start

Authorize a trusted native executable again after it changes or after restore.
On Windows select the actual `.exe`, not `.cmd`/`.bat`. Authenticate outside Forma.
A changed task, settings, HEAD, executable or expired preview needs a new preview
and approval. Dirty/unborn/detached repositories are rejected. Queues require an
explicit Start; restarting never resumes execution. Whole-provider isolation is
unavailable and enabling its requirement blocks execution. Check configured
shared process budget and independently approve tests. Do not loosen permissions
or disable containment to mask a failure.

## Cleanup is blocked

Stop execution/tests and review the result. Preserve tracked, untracked and
ignored changes outside the worktree before retrying. Unknown paths, symlinks,
changed ownership or registration, and unavailable scans require manual review.
Forma deliberately refuses force cleanup. Keep historical branches if needed;
accepting/rejecting a result alone does not make dirty files disposable.

## Report a beta defect

Use the bug template and state which installer/UI actions were actually tried.
Attach only sanitized screenshots with sample data. Signing credentials and
physical Windows/macOS access cannot be inferred from Linux or CI build results.
Known platform limits and candidate artifacts are in [M07](../M07.md).
