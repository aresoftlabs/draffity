import { describe, expect, it } from 'vitest';
import { detectLinguistic, type LinguisticCategory } from './linguistic-focus';

const ALL: LinguisticCategory[] = ['adverb', 'passive', 'dialogue'];
const opts = (categories = ALL, extraWords: string[] = []) => ({ categories, extraWords });

function categoriesIn(text: string, o = opts()) {
  return detectLinguistic(text, o).map((m) => m.category);
}

function matchedText(text: string, o = opts()) {
  return detectLinguistic(text, o).map((m) => text.slice(m.from, m.to));
}

describe('detectLinguistic', () => {
  it('returns nothing for empty text', () => {
    expect(detectLinguistic('', opts())).toEqual([]);
  });

  it('flags Spanish -mente and English -ly adverbs', () => {
    expect(matchedText('Caminó lentamente y habló suddenly.')).toEqual(['lentamente', 'suddenly']);
  });

  it('flags passive voice in Spanish and English (regular participles)', () => {
    // The heuristic is conservative — it covers regular participles
    // (-ado/-ido, -ed/-en), not irregulars like "escrita".
    expect(categoriesIn('La carta fue enviada ayer.')).toContain('passive');
    expect(categoriesIn('The door was opened.')).toContain('passive');
  });

  it('flags dialogue in guillemets and quotes', () => {
    expect(matchedText('Dijo «hola» y "adiós".', opts(['dialogue']))).toEqual([
      '«hola»',
      '"adiós"',
    ]);
  });

  it('honours the category filter', () => {
    const onlyAdverb = detectLinguistic('fue escrita rápidamente «hola»', opts(['adverb']));
    expect(onlyAdverb.every((m) => m.category === 'adverb')).toBe(true);
    expect(onlyAdverb.length).toBe(1);
  });

  it('flags user-configured extra words as adverbs (J-07)', () => {
    const m = detectLinguistic('Era muy bueno.', opts(['adverb'], ['muy']));
    expect(m.map((x) => x.category)).toEqual(['adverb']);
    expect('Era muy bueno.'.slice(m[0].from, m[0].to)).toBe('muy');
  });
});
