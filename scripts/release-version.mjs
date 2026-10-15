import { readFileSync } from 'node:fs';

const read = (path) =>
  readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const version = JSON.parse(read('package.json')).version;
const config = JSON.parse(read('src-tauri/tauri.conf.json'));
const rust = read('src-tauri/Cargo.toml').match(/^version = "([^"]+)"/m)?.[1];
const locked = read('src-tauri/Cargo.lock').match(
  /name = "forma"\nversion = "([^"]+)"/,
)?.[1];
if (!/^\d+\.\d+\.\d+(?:-beta\.\d+)?$/.test(version))
  throw Error('Unsupported release version');
if ([config.version, rust, locked].some((value) => value !== version))
  throw Error('Release versions disagree');
if (process.env.RELEASE_TAG && process.env.RELEASE_TAG !== `v${version}`)
  throw Error('Tag/version mismatch');
if (!read('CHANGELOG.md').includes(`## [${version}]`))
  throw Error('Missing changelog entry');
if (!read(`docs/releases/v${version}.md`).includes(`Forma v${version}`))
  throw Error('Missing release notes');
// Native installers need numeric versions. Review these mappings at every release.
const [base, beta] = version.split('-beta.');
const [major, minor, patch] = base.split('.').map(Number);
const build = beta ? Number(beta) : 9999;
if (config.bundle.windows.wix.version !== `${base}.${build}`)
  throw Error('MSI version mapping disagrees');
if (
  config.bundle.macOS.bundleVersion !==
  `${major}.${minor}.${patch * 10000 + build}`
)
  throw Error('Apple build version mapping disagrees');
console.log(
  `Validated v${version}; MSI ${config.bundle.windows.wix.version}; Apple ${config.bundle.macOS.bundleVersion}`,
);
