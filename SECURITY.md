# Security policy

M01 (0.1.x) is pre-release software. Security fixes target the latest development
revision; there are no long-term support branches yet.

Please use the repository's GitHub **Security → Report a vulnerability** option
when enabled. If it is unavailable, open a public issue asking the maintainers to
enable private reporting without including exploit details, secrets, or personal
data. No private reporting address has been established yet.

Describe the affected revision, platform, reproduction steps, impact, and any
suggested mitigation. Do not upload production databases.

Forma stores unencrypted data in the current user's application data directory.
OS permissions and device security protect that data. The frontend has no
filesystem, shell, HTTP, or SQL plugin. Rust exposes a fixed command set with
validated inputs and parameterized queries. Remote content is not loaded.
The CSP restricts script execution and IPC origins. Release installers are not
yet trusted-signed; see README for distribution limitations.
