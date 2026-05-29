import { NAME_ORIGINS, type NameGender, type NameOrigin } from '@/data/names';

/** Pool of candidate names for an origin + gender. `unisex` always augments
 *  the masc/fem pools; selecting "unisex" returns only the unisex names. */
export function poolFor(origin: NameOrigin, gender: NameGender): string[] {
  if (gender === 'unisex') return [...origin.unisex];
  const base = gender === 'masc' ? origin.masc : origin.fem;
  return [...base, ...origin.unisex];
}

/**
 * Pick up to `count` distinct names from `pool` using `rng` (defaults to
 * `Math.random`). Pure given a deterministic `rng`, so it unit-tests without
 * randomness. Returns fewer than `count` only when the pool is smaller.
 */
export function pickNames(
  pool: string[],
  count: number,
  rng: () => number = Math.random,
): string[] {
  const remaining = [...pool];
  const out: string[] = [];
  const n = Math.min(count, remaining.length);
  for (let i = 0; i < n; i++) {
    const idx = Math.floor(rng() * remaining.length) % remaining.length;
    out.push(remaining.splice(idx, 1)[0]);
  }
  return out;
}

/** Generate names for a built-in origin id + gender. Empty when id unknown. */
export function generateNames(
  originId: string,
  gender: NameGender,
  count: number,
  rng: () => number = Math.random,
): string[] {
  const origin = NAME_ORIGINS.find((o) => o.id === originId);
  if (!origin) return [];
  return pickNames(poolFor(origin, gender), count, rng);
}
