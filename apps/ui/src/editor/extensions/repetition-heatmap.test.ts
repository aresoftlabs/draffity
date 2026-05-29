import { describe, expect, it } from 'vitest';
import { detectRepetitions, DEFAULT_REPETITION_OPTIONS } from './repetition-heatmap';

const opts = DEFAULT_REPETITION_OPTIONS;

function matchedText(text: string) {
  return detectRepetitions(text, opts).map((m) => text.slice(m.from, m.to));
}

describe('detectRepetitions', () => {
  it('returns nothing for empty text', () => {
    expect(detectRepetitions('', opts)).toEqual([]);
  });

  it('flags a content word repeated at/above the threshold', () => {
    // "bosque" ×4 (default wordThreshold = 4).
    const found = matchedText('bosque bosque bosque bosque');
    expect(found.filter((w) => w === 'bosque').length).toBe(4);
  });

  it('does not flag a word below the threshold', () => {
    expect(detectRepetitions('bosque bosque bosque', opts)).toEqual([]);
  });

  it('does not flag common stop words as single repetitions', () => {
    // "para" is a stopword; even ×5 it should not be flagged as a single word.
    const matches = detectRepetitions('para para para para para', opts);
    expect(matches).toEqual([]);
  });

  it('flags a repeated two-word phrase like "she said"', () => {
    const text = Array(8).fill('she said').join(' ');
    const matches = detectRepetitions(text, opts);
    // The phrase span "she said" must appear among the matches.
    expect(matches.some((m) => text.slice(m.from, m.to) === 'she said')).toBe(true);
  });

  it('assigns higher heat levels to more frequent repetitions', () => {
    const text = Array(8).fill('bosque').join(' ');
    const levels = detectRepetitions(text, opts).map((m) => m.level);
    // 8 ≥ wordThreshold*2 → level 3.
    expect(levels.every((l) => l === 3)).toBe(true);
  });
});
