# Private beta bug report template

Copy into the agreed private channel. Use sample data; do not submit a report
automatically or assume GitHub issues are private. Suspected vulnerabilities go
through [SECURITY.md](../SECURITY.md). Do not attach credentials, private
repository paths/code, raw backups/databases, complete logs or memory dumps.

```text
Title:
Tester alias (optional):
Acceptance row ID / blocker ID (if known):
Observed UTC date/time:
Forma version and candidate source SHA:
OS edition and exact version/build:
CPU architecture:
Installer filename and format:
Installer SHA-256 (all 64 characters):
Checksum matched coordinator's reviewed list? Yes / No / Not checked:
Clean install or upgrade (previous version + hash):
Install scope / path with username replaced by <USER>:
Runtime: WebKitGTK + desktop/GPU/display backend, WebView2, or macOS point version:

Reproduction with disposable sample data:
1.
2.
3.
Expected result:
Actual result and sanitized error:
Frequency (e.g. 2 of 3 attempts; don't repeat a destructive failure):
Impact (blocked install / crash / data loss / unexpected execution / UI / other):
First launch / restart / shutdown outcome:
Platform trust prompt text (sanitized; no override):
Workaround tried without changing security controls, or none:
Minimal sanitized log excerpt (optional; plain text only):
Sanitized screenshot (optional; inspect all visible content first):
What remains untested / blocked:
```

Manually replace usernames, home/repository paths, project/customer names, task
text, email addresses, remote URLs and identifying machine names with consistent
placeholders. Remove tokens, authentication headers, keys and command arguments
containing secrets entirely. Keep only a few relevant error lines; built-in
redaction is incomplete. If safe sanitization is uncertain, omit the attachment
and describe the symptom. Never collect a whole home directory, environment dump,
provider transcript, private repository or raw SQLite file to diagnose a report.
