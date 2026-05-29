/**
 * Line-level diff via classic LCS. Good enough for snapshot comparisons —
 * a manuscript chapter rarely exceeds a few hundred lines, so the O(n*m)
 * table fits in memory and runs in a single tick. The output is a flat
 * sequence of ops the UI can render side-by-side.
 *
 * No external dep: keeping the diff in-tree keeps the bundle small and
 * avoids a runtime cost (`diff-match-patch` is ~50 KB minified).
 */

export type DiffKind = 'same' | 'add' | 'remove';

export interface DiffOp {
  kind: DiffKind;
  /** Line from the "before" side. Present for `same` and `remove`. */
  before?: string;
  /** Line from the "after" side. Present for `same` and `add`. */
  after?: string;
}

export function lineDiff(beforeText: string, afterText: string): DiffOp[] {
  const before = splitLines(beforeText);
  const after = splitLines(afterText);
  const m = before.length;
  const n = after.length;
  // Edge cases avoid allocating the LCS table for nothing.
  if (m === 0 && n === 0) return [];
  if (m === 0) return after.map((line) => ({ kind: 'add' as const, after: line }));
  if (n === 0) return before.map((line) => ({ kind: 'remove' as const, before: line }));

  // dp[i][j] = LCS length for before[0..i] and after[0..j]
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (before[i - 1] === after[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  // Backtrack to build the op list in forward order.
  const ops: DiffOp[] = [];
  let i = m;
  let j = n;
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && before[i - 1] === after[j - 1]) {
      ops.push({ kind: 'same', before: before[i - 1], after: after[j - 1] });
      i--;
      j--;
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      ops.push({ kind: 'add', after: after[j - 1] });
      j--;
    } else {
      ops.push({ kind: 'remove', before: before[i - 1] });
      i--;
    }
  }
  ops.reverse();
  return ops;
}

/** Strip HTML tags + collapse whitespace per line. The editor stores
 *  paragraphs as `<p>…</p>` so we get clean per-paragraph lines for free
 *  after normalisation. */
export function htmlToLines(html: string): string {
  if (!html) return '';
  // Insert a line break before block-level openings so the resulting
  // text has one paragraph per line.
  const withBreaks = html
    .replace(/<(p|h[1-6]|li|blockquote|hr)([\s>])/gi, '\n<$1$2')
    .replace(/<br\s*\/?>/gi, '\n');
  // Strip tags after the marker insertion.
  const text = withBreaks.replace(/<[^>]*>/g, '');
  return decodeEntities(text)
    .split(/\r?\n/)
    .map((line) => line.replace(/\s+/g, ' ').trim())
    .filter((line) => line.length > 0)
    .join('\n');
}

function splitLines(text: string): string[] {
  // `''.split('\n')` returns `['']` — one empty line — which would make
  // the diff treat empty input as "one removed empty line". That's never
  // what callers want, so collapse it to a real empty list here.
  if (text.length === 0) return [];
  return text.split(/\r?\n/);
}

function decodeEntities(s: string): string {
  return s
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&nbsp;/g, ' ');
}
