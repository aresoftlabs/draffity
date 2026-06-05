#!/usr/bin/env node
/**
 * sync-voice-manifest.mjs
 *
 * Fetches Piper's official voices.json from HuggingFace and emits our
 * normalized manifest.json (schema v1) to stdout.
 *
 * Usage:
 *   node scripts/sync-voice-manifest.mjs > manifest.json
 *
 * Then publish to R2:
 *   aws s3 cp manifest.json s3://bins-draffity/voices/v1/manifest.json \
 *     --endpoint-url "$R2_ENDPOINT"
 *
 * Requires Node >= 20 (native fetch).
 */

const PIPER_VOICES_URL = 'https://huggingface.co/rhasspy/piper-voices/resolve/main/voices.json';

const HF_BASE = 'https://huggingface.co/rhasspy/piper-voices/resolve/main';

/** Voices to mark as recommended (curated small set). */
const RECOMMENDED = new Set(['es_ES-davefx-medium', 'en_US-amy-medium', 'pt_BR-faber-medium']);

/** Language families to feature first in the UI. */
const FEATURED = ['es', 'en', 'pt'];

/**
 * Capitalizes the first letter of a string.
 * "davefx" → "Davefx", "amy" → "Amy"
 */
function capitalize(str) {
  if (!str) return str;
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * From a Piper entry's `files` object, find the path matching a suffix.
 * Returns { path, size_bytes, md5_digest } or null.
 */
function findFile(files, suffix) {
  for (const [path, meta] of Object.entries(files)) {
    if (path.endsWith(suffix)) {
      return { path, ...meta };
    }
  }
  return null;
}

async function main() {
  const resp = await fetch(PIPER_VOICES_URL);
  if (!resp.ok) {
    throw new Error(`Failed to fetch voices.json: ${resp.status} ${resp.statusText}`);
  }

  /** @type {Record<string, PiperEntry>} */
  const raw = await resp.json();

  const voices = [];

  for (const [id, entry] of Object.entries(raw)) {
    const lang = entry.language ?? {};
    const files = entry.files ?? {};

    const onnxFile = findFile(files, '.onnx');
    const configFile = findFile(files, '.onnx.json');

    // Skip entries that lack BOTH an .onnx model and its .onnx.json config —
    // the Rust parser requires both fields as non-optional strings; a single
    // null url would cause serde to reject the entire manifest.
    if (!onnxFile || !configFile) continue;

    const sizeMb = onnxFile.size_bytes
      ? Math.round((onnxFile.size_bytes / 1_048_576) * 10) / 10
      : null;

    const voice = {
      id,
      name: capitalize(entry.name ?? id),
      lang: lang.family ?? '',
      langName: lang.name_native ?? lang.name_english ?? '',
      locale: lang.code ?? '',
      quality: entry.quality ?? '',
      sizeMb,
      onnxUrl: `${HF_BASE}/${onnxFile.path}`,
      configUrl: `${HF_BASE}/${configFile.path}`,
      onnxMd5: onnxFile.md5_digest ?? null,
      configMd5: configFile.md5_digest ?? null,
      recommended: RECOMMENDED.has(id),
    };

    voices.push(voice);
  }

  const manifest = {
    schemaVersion: 1,
    featured: FEATURED,
    voices,
  };

  process.stdout.write(JSON.stringify(manifest, null, 2) + '\n');
}

main().catch((err) => {
  process.stderr.write(`Error: ${err.message}\n`);
  process.exit(1);
});
