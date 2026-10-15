#!/usr/bin/env python3
"""Collect a complete target's installers or verify the complete four-target RC.

No release/network side effects. Manifests bind each installer to its source SHA,
version, target, size and checksum. Headers reject wrong-format artifacts; native
app headers independently check CPU architecture before collection.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import struct

TARGETS = {
    'x86_64-unknown-linux-gnu': ('appimage/*.AppImage', 'deb/*.deb'),
    'x86_64-pc-windows-msvc': ('nsis/*.exe', 'msi/*.msi'),
    'x86_64-apple-darwin': ('dmg/*.dmg',),
    'aarch64-apple-darwin': ('dmg/*.dmg',),
}
EXTENSIONS = ('.AppImage', '.deb', '.exe', '.msi', '.dmg')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def check_header(path):
    with path.open('rb') as stream:
        head = stream.read(16)
        stream.seek(-512, 2)
        trailer = stream.read(4)
    valid = {
        '.AppImage': head[:4] == b'\x7fELF' and head[8:11] in (b'AI\x01', b'AI\x02'),
        '.deb': head[:8] == b'!<arch>\n',
        '.exe': head[:2] == b'MZ',
        '.msi': head[:8] == bytes.fromhex('d0cf11e0a1b11ae1'),
        '.dmg': trailer == b'koly',
    }
    require(valid.get(path.suffix, False), f'Invalid package format: {path}')


def check_cpu(bundle, target):
    if target.endswith('linux-gnu'):
        binary = next(bundle.glob('appimage/*.AppImage'))
        with binary.open('rb') as stream:
            head = stream.read(20)
        require(head[:6] == b'\x7fELF\x02\x01' and struct.unpack_from('<H', head, 18)[0] == 62, 'Expected Linux x86_64 ELF')
    elif target.endswith('windows-msvc'):
        with (bundle.parent / 'forma.exe').open('rb') as stream:
            stream.seek(60)
            offset = struct.unpack('<I', stream.read(4))[0]
            stream.seek(offset)
            require(stream.read(6) == b'PE\0\0\x64\x86', 'Expected Windows x64 application')
    else:
        with (bundle / 'macos/Forma.app/Contents/MacOS/forma').open('rb') as stream:
            header = stream.read(8)
        cpu = 0x0100000c if target.startswith('aarch64') else 0x01000007
        require(header == struct.pack('<II', 0xfeedfacf, cpu), 'Unexpected macOS application CPU')


def collect(bundle, target, output, version, commit):
    require(target in TARGETS, 'Unknown target')
    require(re.fullmatch(r'[0-9a-f]{40}', commit), 'Expected full source commit SHA')
    require(not output.exists(), 'Output must be a new directory')
    sources = []
    for pattern in TARGETS[target]:
        matches = list(bundle.glob(pattern))
        require(len(matches) == 1, f'Expected exactly one {pattern}, found {len(matches)}')
        source = matches[0]
        require(version in source.name, f'Wrong package version: {source.name}')
        check_header(source)
        sources.append(source)
    check_cpu(bundle, target)
    output.mkdir(parents=True)
    packages = []
    for source in sources:
        name = f'Forma_{version}_{target}{source.suffix}'
        destination = output / name
        shutil.copyfile(source, destination)
        packages.append({'file': name, 'bytes': destination.stat().st_size, 'sha256': digest(destination)})
    manifest = {'version': version, 'commit': commit, 'target': target, 'packages': packages}
    (output / f'{target}.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (output / f'{target}.sha256').write_text(''.join(f"{p['sha256']}  {p['file']}\n" for p in packages))
    print(json.dumps(manifest, indent=2))


def verify(root, version, commit):
    manifests = list(root.rglob('*.json'))
    require(len(manifests) == 4, 'Expected exactly four target manifests')
    targets = set()
    records = []
    for path in manifests:
        manifest = json.loads(path.read_text())
        target = manifest['target']
        require(target in TARGETS and target not in targets, 'Unknown or duplicate target')
        targets.add(target)
        require(manifest['version'] == version and manifest['commit'] == commit, 'Mixed source/version artifacts')
        packages = manifest['packages']
        require(len(packages) == len(TARGETS[target]), 'Incomplete package inventory')
        expected_extensions = {Path(pattern).suffix for pattern in TARGETS[target]}
        require({Path(p['file']).suffix for p in packages} == expected_extensions, 'Wrong package types')
        expected_lines = ''
        for package in packages:
            name = package['file']
            require(name == f'Forma_{version}_{target}{Path(name).suffix}', 'Noncanonical artifact name')
            artifact = path.parent / name
            require(artifact.stat().st_size == package['bytes'] and digest(artifact) == package['sha256'], f'Checksum/size mismatch: {name}')
            check_header(artifact)
            expected_lines += f"{package['sha256']}  {name}\n"
            records.append((name, package['sha256']))
        require((path.parent / f'{target}.sha256').read_text() == expected_lines, 'Checksum sidecar mismatch')
    require(targets == set(TARGETS), 'Missing platform')
    actual = [p for p in root.rglob('*') if p.is_file() and p.suffix in EXTENSIONS]
    require(len(actual) == 6, 'Unexpected or missing installers')
    (root / 'SHA256SUMS').write_text(''.join(f'{sha}  {name}\n' for name, sha in sorted(records)))
    print(f'PASS: six installers, four targets, version {version}, source {commit}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['collect', 'verify'])
    parser.add_argument('root', type=Path)
    parser.add_argument('--target', choices=TARGETS)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--version', required=True)
    parser.add_argument('--commit', required=True)
    args = parser.parse_args()
    if args.mode == 'collect':
        require(args.target and args.output, 'collect needs --target and --output')
        collect(args.root, args.target, args.output, args.version, args.commit)
    else:
        verify(args.root, args.version, args.commit)
