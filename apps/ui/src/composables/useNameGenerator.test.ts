import { describe, expect, it } from 'vitest';
import { generateNames, pickNames, poolFor } from './useNameGenerator';
import { NAME_ORIGINS } from '@/data/names';

/** Deterministic RNG cycling through fixed fractions. */
function seqRng(values: number[]): () => number {
  let i = 0;
  return () => values[i++ % values.length];
}

describe('name generator', () => {
  it('poolFor merges unisex into masc/fem but isolates unisex', () => {
    const celtic = NAME_ORIGINS.find((o) => o.id === 'celtic')!;
    expect(poolFor(celtic, 'masc')).toEqual([...celtic.masc, ...celtic.unisex]);
    expect(poolFor(celtic, 'unisex')).toEqual(celtic.unisex);
  });

  it('pickNames returns distinct names and never more than the pool', () => {
    const pool = ['a', 'b', 'c'];
    const picked = pickNames(pool, 10, seqRng([0]));
    expect(picked.length).toBe(3);
    expect(new Set(picked).size).toBe(3);
  });

  it('pickNames is deterministic given a fixed rng', () => {
    const pool = ['a', 'b', 'c', 'd'];
    // rng=0 always picks the current first element.
    expect(pickNames(pool, 2, seqRng([0]))).toEqual(['a', 'b']);
  });

  it('generateNames returns [] for an unknown origin', () => {
    expect(generateNames('nope', 'masc', 5)).toEqual([]);
  });

  it('generateNames pulls from the requested origin', () => {
    const out = generateNames('norse', 'fem', 3, seqRng([0]));
    const norse = NAME_ORIGINS.find((o) => o.id === 'norse')!;
    expect(out.every((n) => poolFor(norse, 'fem').includes(n))).toBe(true);
    expect(out.length).toBe(3);
  });

  it('ships a substantial built-in dataset', () => {
    expect(NAME_ORIGINS.length).toBeGreaterThanOrEqual(10);
    for (const o of NAME_ORIGINS) {
      expect(o.masc.length + o.fem.length + o.unisex.length).toBeGreaterThan(10);
    }
  });
});
