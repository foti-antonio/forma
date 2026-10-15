# Working on Forma

- Read README.md, docs/ARCHITECTURE.md, and the current milestone before changes.
- Stack: Tauri 2, Rust, strict TypeScript, React/Vite, Tailwind, local shadcn-style
  Radix primitives, Zustand, bundled SQLite, pnpm.
- Preserve the UI → store → typed IPC → command → service → repository boundary.
- Rust owns persistence. Never replace it with browser/mock persistence.
- Validate backend inputs, use structured errors, and parameterize SQL.
- Use app-data paths for production databases; never commit databases or secrets.
- Keep capabilities minimal. Do not add shell/filesystem/network access casually.
- Use UUIDs, UTC timestamps, foreign keys, and versioned transactional migrations.
- Keep migrations append-only after release; test upgrade and restart behavior.
- Follow README's formatting, lint, type, frontend/Rust test and clippy commands.
  Service tests can use --no-default-features without WebKit system dependencies.
- Use accessible controls, real error/loading/empty states, and deletion confirmation.
- Document commands and actual results in the milestone report. Identify every
  unverified platform and never claim tests ran without recorded execution.
- M01 excludes issue UI, AI agents, cloud synchronization, and authentication.
