#!/usr/bin/env node
/**
 * build-update-manifest.mjs
 *
 * Two responsibilities, selected by subcommand:
 *   fragment  — read a single installer's .sig and emit a per-platform fragment
 *               { key, url, signature } as JSON.
 *   assemble  — read all fragments in a directory and emit the Tauri updater
 *               manifest (latest.json) for stable.
 *
 * Used by .github/workflows/release.yml. The pure `buildManifest` is unit-tested.
 * Requires Node >= 20.
 */
import { readFileSync, writeFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Pure: build the Tauri updater manifest object.
 * @param {{version: string, notes?: string, pubDate: string, platforms: {key: string, url: string, signature: string}[]}} input
 */
export function buildManifest({ version, notes, pubDate, platforms }) {
  if (!version) throw new Error('version is required');
  if (!Array.isArray(platforms) || platforms.length === 0) {
    throw new Error('at least one platform fragment is required');
  }
  const out = { version, notes: notes ?? '', pub_date: pubDate, platforms: {} };
  for (const p of platforms) {
    if (!p.key) throw new Error('fragment missing "key"');
    if (!p.url) throw new Error(`fragment ${p.key} missing "url"`);
    if (!p.signature) throw new Error(`fragment ${p.key} missing "signature"`);
    out.platforms[p.key] = { signature: p.signature, url: p.url };
  }
  return out;
}

/** Read every *.json fragment in `dir` into an array of objects. */
export function loadFragments(dir) {
  return readdirSync(dir)
    .filter((f) => f.endsWith('.json'))
    .map((f) => JSON.parse(readFileSync(join(dir, f), 'utf8')));
}

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 2) {
    args[argv[i].replace(/^--/, '')] = argv[i + 1];
  }
  return args;
}

function cmdFragment(args) {
  const signature = readFileSync(args['sig-file'], 'utf8').trim();
  const fragment = { key: args.key, url: args.url, signature };
  writeFileSync(args.out, JSON.stringify(fragment, null, 2) + '\n');
}

function cmdAssemble(args) {
  const manifest = buildManifest({
    version: args.version,
    notes: args.notes,
    pubDate: args['pub-date'],
    platforms: loadFragments(args.fragments),
  });
  writeFileSync(args.out, JSON.stringify(manifest, null, 2) + '\n');
}

function main() {
  const [cmd, ...rest] = process.argv.slice(2);
  const args = parseArgs(rest);
  if (cmd === 'fragment') cmdFragment(args);
  else if (cmd === 'assemble') cmdAssemble(args);
  else {
    process.stderr.write(`Unknown command: ${cmd ?? '(none)'}\n`);
    process.exit(1);
  }
}

// Run the CLI only when invoked directly, not when imported by tests.
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
