"""Release gates must reject incomplete, mixed and corrupted candidates."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('release', Path(__file__).with_name('release-artifacts.py'))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ArtifactGateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.version = '0.1.0-beta.1'
        self.commit = 'a' * 40
        for target, patterns in release.TARGETS.items():
            packages = []
            for pattern in patterns:
                extension = Path(pattern).suffix
                data = bytearray(1024)
                header = {'.AppImage': b'\x7fELF\x02\x01\0\0AI\x02', '.deb': b'!<arch>\n', '.exe': b'MZ', '.msi': bytes.fromhex('d0cf11e0a1b11ae1'), '.dmg': b''}[extension]
                data[:len(header)] = header
                if extension == '.dmg':
                    data[-512:-508] = b'koly'
                name = f'Forma_{self.version}_{target}{extension}'
                path = self.root / name
                path.write_bytes(data)
                packages.append({'file': name, 'bytes': len(data), 'sha256': release.digest(path)})
            manifest = {'version': self.version, 'commit': self.commit, 'target': target, 'packages': packages}
            (self.root / f'{target}.json').write_text(json.dumps(manifest))
            (self.root / f'{target}.sha256').write_text(''.join(f"{p['sha256']}  {p['file']}\n" for p in packages))

    def verify(self):
        release.verify(self.root, self.version, self.commit)

    def test_complete_inventory(self):
        self.verify()
        self.assertEqual(len((self.root / 'SHA256SUMS').read_text().splitlines()), 6)

    def test_missing_platform(self):
        next(self.root.glob('*.json')).unlink()
        with self.assertRaisesRegex(ValueError, 'four target'):
            self.verify()

    def test_corrupt_installer(self):
        next(self.root.glob('*.msi')).write_bytes(b'corrupt')
        with self.assertRaisesRegex(ValueError, 'Checksum/size'):
            self.verify()

    def test_mixed_commit(self):
        path = next(self.root.glob('*.json'))
        data = json.loads(path.read_text())
        data['commit'] = 'b' * 40
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'Mixed source'):
            self.verify()

    def test_missing_installer_type(self):
        path = self.root / 'x86_64-pc-windows-msvc.json'
        data = json.loads(path.read_text())
        data['packages'].pop()
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'Incomplete'):
            self.verify()

    def test_checksum_sidecar_tampering(self):
        next(self.root.glob('*.sha256')).write_text('invalid')
        with self.assertRaisesRegex(ValueError, 'sidecar'):
            self.verify()

    def test_wrong_header_even_with_matching_digest(self):
        path = self.root / 'aarch64-apple-darwin.json'
        data = json.loads(path.read_text())
        artifact = self.root / data['packages'][0]['file']
        artifact.write_bytes(bytes(1024))
        data['packages'][0]['sha256'] = release.digest(artifact)
        path.write_text(json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'Invalid package format'):
            self.verify()

    def test_unlisted_installer(self):
        (self.root / 'unexpected.exe').write_bytes(b'MZ')
        with self.assertRaisesRegex(ValueError, 'Unexpected or missing'):
            self.verify()
