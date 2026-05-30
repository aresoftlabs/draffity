import { describe, expect, it } from 'vitest';
import { presetOverrides } from './preset';

describe('Draffity PrimeVue preset overrides', () => {
  it('applies the terracotta primary ramp', () => {
    expect(presetOverrides.semantic.primary['500']).toBe('#b5651d');
    expect(presetOverrides.semantic.primary['400']).toBe('#d08a4a');
  });

  it('applies the warm surface ramp to both colour schemes', () => {
    expect(presetOverrides.semantic.colorScheme.light.surface['50']).toBe('#faf7f0');
    expect(presetOverrides.semantic.colorScheme.dark.surface['950']).toBe('#1b1813');
  });
});
