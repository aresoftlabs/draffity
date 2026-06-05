import { test } from 'node:test';
import assert from 'node:assert/strict';
import { buildManifest } from './build-update-manifest.mjs';

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
