import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';
import type { Node as ProseMirrorNode } from '@tiptap/pm/model';

/**
 * Repetition heatmap (J-08): a toggleable, local (no-AI) overlay that flags
 * over-used content words and repeated two-word phrases. A fast subset of the
 * AI repetition validator (G-06). Local, no-server overlay. Implemented as
 * ProseMirror decorations — it never mutates the document.
 *
 * Detection is pure (`detectRepetitions`) and unit-tested without an editor.
 */

export interface RepetitionMatch {
  from: number;
  to: number;
  /** Heat level 1–3 (higher = more repetitions). */
  level: 1 | 2 | 3;
}

export interface RepetitionOptions {
  /** Minimum length for a single word to be considered (skips short words). */
  minWordLen: number;
  /** Times a word must repeat to be flagged. */
  wordThreshold: number;
  /** Times a two-word phrase must repeat to be flagged. */
  phraseThreshold: number;
}

export const DEFAULT_REPETITION_OPTIONS: RepetitionOptions = {
  minWordLen: 4,
  wordThreshold: 4,
  phraseThreshold: 3,
};

// Common ES + EN function words — repeating these is expected, not a smell.
const STOPWORDS = new Set([
  // Spanish
  'para',
  'pero',
  'como',
  'porque',
  'cuando',
  'donde',
  'entre',
  'sobre',
  'desde',
  'hasta',
  'este',
  'esta',
  'esto',
  'esos',
  'esas',
  'aquel',
  'todo',
  'toda',
  'todos',
  'todas',
  'cada',
  'unos',
  'unas',
  'muy',
  'mas',
  'más',
  'sus',
  'con',
  'sin',
  'los',
  'las',
  'una',
  'que',
  // English
  'the',
  'and',
  'that',
  'this',
  'with',
  'from',
  'have',
  'has',
  'had',
  'was',
  'were',
  'are',
  'for',
  'but',
  'not',
  'they',
  'them',
  'their',
  'there',
  'then',
  'than',
  'into',
  'your',
  'you',
  'his',
  'her',
  'she',
  'him',
  'its',
  'our',
  'who',
  'what',
  'when',
  'which',
  'will',
]);

interface Token {
  word: string;
  from: number;
  to: number;
}

const WORD_RE = /[\p{L}][\p{L}'’-]*/gu;

function tokenize(text: string): Token[] {
  const tokens: Token[] = [];
  WORD_RE.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = WORD_RE.exec(text)) !== null) {
    tokens.push({ word: m[0].toLowerCase(), from: m.index, to: m.index + m[0].length });
  }
  return tokens;
}

function level(count: number, threshold: number): 1 | 2 | 3 {
  if (count >= threshold * 2) return 3;
  if (count >= Math.ceil(threshold * 1.5)) return 2;
  return 1;
}

/** Find repeated content words and two-word phrases in `text`. Pure. */
export function detectRepetitions(text: string, opts: RepetitionOptions): RepetitionMatch[] {
  if (!text) return [];
  const tokens = tokenize(text);
  if (tokens.length === 0) return [];

  // Single-word frequencies (content words only).
  const wordCount = new Map<string, number>();
  for (const tk of tokens) {
    if (tk.word.length < opts.minWordLen || STOPWORDS.has(tk.word)) continue;
    wordCount.set(tk.word, (wordCount.get(tk.word) ?? 0) + 1);
  }

  // Two-word phrase frequencies (skip pairs where both words are stopwords).
  const phraseCount = new Map<string, number>();
  for (let i = 0; i < tokens.length - 1; i++) {
    const a = tokens[i];
    const b = tokens[i + 1];
    if (STOPWORDS.has(a.word) && STOPWORDS.has(b.word)) continue;
    const key = `${a.word} ${b.word}`;
    phraseCount.set(key, (phraseCount.get(key) ?? 0) + 1);
  }

  const matches: RepetitionMatch[] = [];

  // Phrase matches first (they span two tokens; take precedence visually).
  for (let i = 0; i < tokens.length - 1; i++) {
    const a = tokens[i];
    const b = tokens[i + 1];
    if (STOPWORDS.has(a.word) && STOPWORDS.has(b.word)) continue;
    const count = phraseCount.get(`${a.word} ${b.word}`) ?? 0;
    if (count >= opts.phraseThreshold) {
      matches.push({ from: a.from, to: b.to, level: level(count, opts.phraseThreshold) });
    }
  }

  // Single-word matches.
  for (const tk of tokens) {
    const count = wordCount.get(tk.word) ?? 0;
    if (count >= opts.wordThreshold) {
      matches.push({ from: tk.from, to: tk.to, level: level(count, opts.wordThreshold) });
    }
  }

  return matches;
}

export const repetitionKey = new PluginKey<RepetitionPluginState>('repetitionHeatmap');

interface RepetitionPluginState {
  enabled: boolean;
  options: RepetitionOptions;
}

function buildDecorations(doc: ProseMirrorNode, options: RepetitionOptions): DecorationSet {
  const decos: Decoration[] = [];
  doc.descendants((node, pos) => {
    if (!node.isText || !node.text) return;
    for (const m of detectRepetitions(node.text, options)) {
      decos.push(Decoration.inline(pos + m.from, pos + m.to, { class: `rep-heat-${m.level}` }));
    }
  });
  return DecorationSet.create(doc, decos);
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    repetitionHeatmap: {
      setRepetitionHeatmap: (enabled: boolean, options?: Partial<RepetitionOptions>) => ReturnType;
    };
  }
}

export const RepetitionHeatmap = Extension.create({
  name: 'repetitionHeatmap',

  addCommands() {
    return {
      setRepetitionHeatmap:
        (enabled, options) =>
        ({ state, dispatch }) => {
          if (dispatch) {
            const prev = repetitionKey.getState(state);
            const merged: RepetitionOptions = {
              ...DEFAULT_REPETITION_OPTIONS,
              ...prev?.options,
              ...options,
            };
            dispatch(state.tr.setMeta(repetitionKey, { enabled, options: merged }));
          }
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin<RepetitionPluginState>({
        key: repetitionKey,
        state: {
          init: () => ({ enabled: false, options: DEFAULT_REPETITION_OPTIONS }),
          apply(tr, value) {
            const meta = tr.getMeta(repetitionKey) as RepetitionPluginState | undefined;
            return meta ?? value;
          },
        },
        props: {
          decorations(state) {
            const s = repetitionKey.getState(state);
            if (!s?.enabled) return null;
            return buildDecorations(state.doc, s.options);
          },
        },
      }),
    ];
  },
});
