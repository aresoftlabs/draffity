import { describe, expect, it } from 'vitest';
import { Schema } from '@tiptap/pm/model';
import { findMatches } from './useProseMirrorSearch';

// Minimal schema: a `doc` of `paragraph` blocks containing inline text. The
// real TipTap schema is richer (headings, lists, marks, custom nodes…),
// but `findMatches` only cares about text nodes so this is enough.
const schema = new Schema({
  nodes: {
    doc: { content: 'block+' },
    paragraph: { group: 'block', content: 'inline*' },
    text: { group: 'inline' },
  },
});

function docOf(...paragraphs: string[]) {
  return schema.node(
    'doc',
    null,
    paragraphs.map((p) => schema.node('paragraph', null, schema.text(p))),
  );
}

describe('findMatches', () => {
  it('returns empty for empty query', () => {
    const doc = docOf('Hello world');
    expect(findMatches(doc, '', false)).toEqual([]);
  });

  it('finds a single match (case insensitive)', () => {
    const doc = docOf('Hello world');
    const hits = findMatches(doc, 'world', false);
    expect(hits.length).toBe(1);
    expect(hits[0].to - hits[0].from).toBe(5);
  });

  it('case sensitive search respects casing', () => {
    const doc = docOf('Hello hello HELLO');
    const ci = findMatches(doc, 'hello', false);
    expect(ci.length).toBe(3);
    const cs = findMatches(doc, 'hello', true);
    expect(cs.length).toBe(1);
  });

  it('finds non-overlapping matches across paragraphs', () => {
    const doc = docOf('foo bar foo', 'foo');
    const hits = findMatches(doc, 'foo', false);
    expect(hits.length).toBe(3);
    // Returns ascending positions.
    expect(hits[0].from).toBeLessThan(hits[1].from);
    expect(hits[1].from).toBeLessThan(hits[2].from);
  });

  it('does not overlap consecutive matches', () => {
    // "aaaa" with needle "aa" must return 2, not 3 (no overlap).
    const doc = docOf('aaaa');
    const hits = findMatches(doc, 'aa', false);
    expect(hits.length).toBe(2);
  });
});
