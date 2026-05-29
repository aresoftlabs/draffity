import { describe, it, expect } from 'vitest';
import { htmlToLines, lineDiff } from './useTextDiff';

describe('lineDiff', () => {
  it('returns same ops for identical inputs', () => {
    const ops = lineDiff('a\nb\nc', 'a\nb\nc');
    expect(ops.every((o) => o.kind === 'same')).toBe(true);
    expect(ops.map((o) => o.before)).toEqual(['a', 'b', 'c']);
  });

  it('detects pure insertions', () => {
    const ops = lineDiff('a\nb', 'a\nx\nb');
    const adds = ops.filter((o) => o.kind === 'add');
    expect(adds.map((o) => o.after)).toEqual(['x']);
  });

  it('detects pure removals', () => {
    const ops = lineDiff('a\nb\nc', 'a\nc');
    const removes = ops.filter((o) => o.kind === 'remove');
    expect(removes.map((o) => o.before)).toEqual(['b']);
  });

  it('handles empty before side', () => {
    const ops = lineDiff('', 'a\nb');
    expect(ops.every((o) => o.kind === 'add')).toBe(true);
    expect(ops.map((o) => o.after)).toEqual(['a', 'b']);
  });

  it('handles empty after side', () => {
    const ops = lineDiff('a\nb', '');
    expect(ops.every((o) => o.kind === 'remove')).toBe(true);
    expect(ops.map((o) => o.before)).toEqual(['a', 'b']);
  });
});

describe('htmlToLines', () => {
  it('produces one line per paragraph', () => {
    const lines = htmlToLines('<p>Hola</p><p>Mundo</p>');
    expect(lines.split('\n')).toEqual(['Hola', 'Mundo']);
  });

  it('decodes entities and strips inline markup', () => {
    const lines = htmlToLines('<p>Hola &amp; <strong>mundo</strong></p>');
    expect(lines).toBe('Hola & mundo');
  });

  it('treats br as a paragraph break', () => {
    const lines = htmlToLines('<p>uno<br/>dos</p>');
    expect(lines.split('\n')).toEqual(['uno', 'dos']);
  });

  it('returns empty for empty html', () => {
    expect(htmlToLines('')).toBe('');
    expect(htmlToLines('<p></p>')).toBe('');
  });
});
