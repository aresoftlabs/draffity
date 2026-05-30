import { describe, expect, it } from 'vitest';
import { WARM_PRIMARY, WARM_SURFACE } from './tokens';

const HEX = /^#[0-9a-f]{6}$/;

describe('warm design tokens', () => {
  it('primary ramp has the 11 standard stops 50..950', () => {
    expect(Object.keys(WARM_PRIMARY)).toEqual([
      '50',
      '100',
      '200',
      '300',
      '400',
      '500',
      '600',
      '700',
      '800',
      '900',
      '950',
    ]);
  });

  it('surface ramp has the 12 standard stops 0..950', () => {
    expect(Object.keys(WARM_SURFACE)).toEqual([
      '0',
      '50',
      '100',
      '200',
      '300',
      '400',
      '500',
      '600',
      '700',
      '800',
      '900',
      '950',
    ]);
  });

  it('anchors the terracotta accent (light=500, dark=400)', () => {
    expect(WARM_PRIMARY['500']).toBe('#b5651d');
    expect(WARM_PRIMARY['400']).toBe('#d08a4a');
  });

  it('anchors the warm paper/charcoal surfaces', () => {
    expect(WARM_SURFACE['0']).toBe('#ffffff');
    expect(WARM_SURFACE['50']).toBe('#faf7f0');
    expect(WARM_SURFACE['950']).toBe('#1b1813');
  });

  it('every stop is a 6-digit lowercase hex', () => {
    for (const v of Object.values(WARM_PRIMARY)) expect(v).toMatch(HEX);
    for (const v of Object.values(WARM_SURFACE)) expect(v).toMatch(HEX);
  });
});
