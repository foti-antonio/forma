import { spawnSync } from 'node:child_process';

// Vitest's workers must also use jsdom's Storage rather than Node's host API.
const result = spawnSync(
  process.execPath,
  ['node_modules/vitest/vitest.mjs', 'run', ...process.argv.slice(2)],
  {
    stdio: 'inherit',
    env: {
      ...process.env,
      NODE_OPTIONS: [process.env.NODE_OPTIONS, '--no-experimental-webstorage']
        .filter(Boolean)
        .join(' '),
    },
  },
);
if (result.error) throw result.error;
process.exit(result.status ?? 1);
