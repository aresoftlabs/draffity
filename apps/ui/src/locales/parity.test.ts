import { describe, expect, it } from 'vitest';
import es from './es.json';
import en from './en.json';
import pt from './pt.json';
import fr from './fr.json';
import itJson from './it.json';

type Json = Record<string, unknown>;

/** Rutas de claves hoja, ordenadas (p. ej. "settings.voiceTitle"). */
function leafKeys(obj: Json, prefix = ''): string[] {
  const out: string[] = [];
  for (const [k, v] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === 'object' && !Array.isArray(v)) out.push(...leafKeys(v as Json, path));
    else out.push(path);
  }
  return out.sort();
}

describe('locale parity', () => {
  const ref = leafKeys(es as Json);
  const others: Record<string, Json> = { en, pt, fr, it: itJson };
  for (const [name, msgs] of Object.entries(others)) {
    it(`${name}.json has exactly the same keys as es.json`, () => {
      expect(leafKeys(msgs)).toEqual(ref);
    });
  }
});
