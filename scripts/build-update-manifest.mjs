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
  if (!pubDate) throw new Error('pubDate is required');
  if (!Array.isArray(platforms) || platforms.length === 0) {
    throw new Error('at least one platform fragment is required');
  }
  const out = { version, notes: notes ?? '', pub_date: pubDate, platforms: {} };
  for (const p of platforms) {
    if (!p.key) throw new Error('fragment missing "key"');
    if (!p.url) throw new Error(`fragment ${p.key} missing "url"`);
    if (!p.signature) throw new Error(`fragment ${p.key} missing "signature"`);
    if (out.platforms[p.key]) throw new Error(`duplicate platform key "${p.key}"`);
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

/** Parse `--flag value` pairs. Flags without a following value are ignored so a
 *  misconfigured workflow can't silently shift every key onto the wrong value. */
function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    const token = argv[i];
    if (!token.startsWith('--')) continue;
    const next = argv[i + 1];
    if (next === undefined || next.startsWith('--')) continue;
    args[token.slice(2)] = next;
    i++;
  }
  return args;
}

/** Fail loudly (non-zero exit, clear message) when a required CLI arg is absent,
 *  rather than letting fs throw an opaque stack trace deep inside a release run. */
function requireArgs(args, names) {
  const missing = names.filter((n) => args[n] === undefined);
  if (missing.length > 0) {
    throw new Error(`missing required argument(s): ${missing.map((n) => `--${n}`).join(', ')}`);
  }
}

function cmdFragment(args) {
  requireArgs(args, ['key', 'url', 'sig-file', 'out']);
  const signature = readFileSync(args['sig-file'], 'utf8').trim();
  const fragment = { key: args.key, url: args.url, signature };
  writeFileSync(args.out, JSON.stringify(fragment, null, 2) + '\n');
}

function cmdAssemble(args) {
  requireArgs(args, ['fragments', 'version', 'pub-date', 'out']);
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
  try {
    if (cmd === 'fragment') cmdFragment(args);
    else if (cmd === 'assemble') cmdAssemble(args);
    else throw new Error(`unknown command: ${cmd ?? '(none)'}`);
  } catch (e) {
    process.stderr.write(`Error: ${e instanceof Error ? e.message : String(e)}\n`);
    process.exit(1);
  }
}

// Run the CLI only when invoked directly, not when imported by tests.
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
