import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { buildManifest, loadFragments } from './build-update-manifest.mjs';

test('buildManifest assembles version, notes, pub_date and platforms map', () => {
  const manifest = buildManifest({
    version: '0.13.0',
    notes: 'Mejoras de exportación.',
    pubDate: '2026-06-05T12:00:00Z',
    platforms: [
      {
        key: 'windows-x86_64',
        url: 'https://bins.draffity.com/app/0.13.0/X_x64-setup.exe',
        signature: 'SIGWIN',
      },
      {
        key: 'linux-x86_64',
        url: 'https://bins.draffity.com/app/0.13.0/X_amd64.AppImage',
        signature: 'SIGLNX',
      },
    ],
  });
  assert.deepEqual(manifest, {
    version: '0.13.0',
    notes: 'Mejoras de exportación.',
    pub_date: '2026-06-05T12:00:00Z',
    platforms: {
      'windows-x86_64': {
        signature: 'SIGWIN',
        url: 'https://bins.draffity.com/app/0.13.0/X_x64-setup.exe',
      },
      'linux-x86_64': {
        signature: 'SIGLNX',
        url: 'https://bins.draffity.com/app/0.13.0/X_amd64.AppImage',
      },
    },
  });
});

test('buildManifest throws when version is missing', () => {
  assert.throws(
    () =>
      buildManifest({
        version: '',
        pubDate: 'x',
        platforms: [{ key: 'k', url: 'u', signature: 's' }],
      }),
    /version is required/,
  );
});

test('buildManifest throws when pubDate is missing', () => {
  assert.throws(
    () =>
      buildManifest({
        version: '0.13.0',
        platforms: [{ key: 'k', url: 'u', signature: 's' }],
      }),
    /pubDate is required/,
  );
});

test('buildManifest throws on a duplicate platform key', () => {
  assert.throws(
    () =>
      buildManifest({
        version: '0.13.0',
        pubDate: 'x',
        platforms: [
          { key: 'windows-x86_64', url: 'u1', signature: 's1' },
          { key: 'windows-x86_64', url: 'u2', signature: 's2' },
        ],
      }),
    /duplicate platform key "windows-x86_64"/,
  );
});

test('buildManifest throws when there are no platforms', () => {
  assert.throws(
    () => buildManifest({ version: '0.13.0', pubDate: 'x', platforms: [] }),
    /at least one platform/,
  );
});

test('buildManifest throws when a fragment is missing a field', () => {
  assert.throws(
    () =>
      buildManifest({
        version: '0.13.0',
        pubDate: 'x',
        platforms: [{ key: 'windows-x86_64', url: 'u' }],
      }),
    /missing "signature"/,
  );
});

test('buildManifest defaults missing notes to empty string', () => {
  const m = buildManifest({
    version: '0.13.0',
    pubDate: 'x',
    platforms: [{ key: 'k', url: 'u', signature: 's' }],
  });
  assert.equal(m.notes, '');
});

test('loadFragments reads every *.json in a directory', () => {
  const dir = mkdtempSync(join(tmpdir(), 'frags-'));
  try {
    writeFileSync(
      join(dir, 'windows-x86_64.json'),
      JSON.stringify({ key: 'windows-x86_64', url: 'u1', signature: 's1' }),
    );
    writeFileSync(
      join(dir, 'linux-x86_64.json'),
      JSON.stringify({ key: 'linux-x86_64', url: 'u2', signature: 's2' }),
    );
    writeFileSync(join(dir, 'ignore.txt'), 'not json');
    const frags = loadFragments(dir).sort((a, b) => a.key.localeCompare(b.key));
    assert.equal(frags.length, 2);
    assert.deepEqual(frags[0], { key: 'linux-x86_64', url: 'u2', signature: 's2' });
    assert.deepEqual(frags[1], { key: 'windows-x86_64', url: 'u1', signature: 's1' });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('fragment subcommand reads a .sig file and writes a fragment json', () => {
  const dir = mkdtempSync(join(tmpdir(), 'frag-cli-'));
  try {
    const sigPath = join(dir, 'app.exe.sig');
    const outPath = join(dir, 'windows-x86_64.json');
    writeFileSync(sigPath, '  SIGCONTENT\n');
    const res = spawnSync(
      process.execPath,
      [
        'scripts/build-update-manifest.mjs',
        'fragment',
        '--key',
        'windows-x86_64',
        '--url',
        'https://bins.draffity.com/app/0.13.0/app.exe',
        '--sig-file',
        sigPath,
        '--out',
        outPath,
      ],
      { encoding: 'utf8' },
    );
    assert.equal(res.status, 0, res.stderr);
    assert.deepEqual(JSON.parse(readFileSync(outPath, 'utf8')), {
      key: 'windows-x86_64',
      url: 'https://bins.draffity.com/app/0.13.0/app.exe',
      signature: 'SIGCONTENT',
    });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
