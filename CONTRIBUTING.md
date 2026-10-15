# Contributing

Follow the platform setup in README.md. Use pnpm and retain both lockfiles.
Keep changes focused, explain the user-facing behavior, and provide the checks
you actually ran in a pull request. Clearly identify platform checks you could
not run.

Run the quality commands in README before submitting. Use `pnpm format` and
`cargo fmt --manifest-path src-tauri/Cargo.toml` to format. Add meaningful tests
for database, validation, state, and behavioral changes.

Frontend features belong under `src/features/`. Keep IPC types in
`src/lib/ipc.ts`; do not access SQLite, shell, or arbitrary files from React.
Backend command handlers delegate to domain services and repositories.
Migrations must be append-only once released, versioned, and transactional.
Never commit real databases, credentials, build output, or personal information.

UI controls must be keyboard accessible, operations must show failures, and
destructive actions require confirmation. Preserve system theme support.
Use the existing Radix/shadcn-style primitives rather than a second component
framework.

Report security issues privately as described in SECURITY.md. By contributing,
you agree to license your contribution under Apache-2.0.
