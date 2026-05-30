import { describe, expect, it } from 'vitest';
import { coverTone, COVER_TONES } from './projectCover';

describe('coverTone', () => {
  it('devuelve siempre un tono de la paleta curada', () => {
    for (const id of ['a', 'proyecto-1', 'xyz', '']) {
      expect(COVER_TONES).toContain(coverTone(id));
    }
  });

  it('es determinístico: mismo id → mismo tono', () => {
    expect(coverTone('el-faro')).toBe(coverTone('el-faro'));
  });

  it('distribuye distintos ids (no todos al mismo tono)', () => {
    const tones = new Set(['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'].map(coverTone));
    expect(tones.size).toBeGreaterThan(1);
  });

  it('todos los tonos son hex de 6 dígitos en minúscula', () => {
    for (const tone of COVER_TONES) expect(tone).toMatch(/^#[0-9a-f]{6}$/);
  });
});
