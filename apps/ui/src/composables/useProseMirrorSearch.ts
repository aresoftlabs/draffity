import type { Node as ProseMirrorNode } from '@tiptap/pm/model';

/** Half-open document range that contains a single match. */
export interface FindMatch {
  from: number;
  to: number;
}

/**
 * Search a ProseMirror document for every occurrence of `query`. Pure:
 * takes a doc + options, returns matches. No editor instance required, so
 * the algorithm is unit-testable without mounting TipTap.
 *
 * Empty / whitespace-only queries return `[]` without walking the doc.
 */
export function findMatches(
  doc: ProseMirrorNode,
  query: string,
  caseSensitive: boolean,
): FindMatch[] {
  if (!query) return [];
  const needle = caseSensitive ? query : query.toLowerCase();
  const found: FindMatch[] = [];
  doc.descendants((node, pos) => {
    if (!node.isText || !node.text) return;
    const haystack = caseSensitive ? node.text : node.text.toLowerCase();
    let idx = 0;
    while ((idx = haystack.indexOf(needle, idx)) !== -1) {
      found.push({ from: pos + idx, to: pos + idx + needle.length });
      idx += needle.length;
    }
  });
  return found;
}
