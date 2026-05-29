import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';
import type { Node as ProseMirrorNode } from '@tiptap/pm/model';

/**
 * Linguistic Focus (J-06): a toggleable, read-only highlight overlay that marks
 * stylistic patterns writers usually want to audit — adverbs (`-mente`/`-ly`),
 * passive-voice constructions (ES `ser/estar + participio`, EN `be + -ed/-en`),
 * and dialogue. Implemented as ProseMirror inline decorations so it never
 * touches the document content (no marks stored, nothing exported).
 *
 * Detection is pure (`detectLinguistic`) and unit-tested without an editor.
 */

export type LinguisticCategory = 'adverb' | 'passive' | 'dialogue';

export interface LinguisticMatch {
  /** Offsets relative to the analysed text. */
  from: number;
  to: number;
  category: LinguisticCategory;
}

export interface LinguisticOptions {
  categories: LinguisticCategory[];
  /** Extra exact words to flag as adverbs (J-07 — user-configurable). */
  extraWords: string[];
}

const ADVERB_RE = /\b[\p{L}]{3,}(?:mente|ly)\b/giu;

// Conservative passive heuristic: an auxiliary/copula followed by a past
// participle. Spanish (`ser/estar` + `-ado/-ido`) and English (`be` + `-ed/-en`).
const PASSIVE_RE =
  /\b(?:soy|eres|es|somos|son|fui|fuiste|fue|fuimos|fueron|era|eras|éramos|eran|será|serán|sido|siendo|está|están|estaba|estaban|is|are|was|were|be|been|being|am)\s+[\p{L}]+(?:ado|ada|ados|adas|ido|ida|idos|idas|ed|en)\b/giu;

// Dialogue: guillemets «…», straight/curly double quotes, or an em-dash run to
// end of line (Spanish dialogue dash).
const DIALOGUE_RE = /«[^»]*»|"[^"]*"|“[^”]*”|—[^—\n]*/gu;

function pushAll(re: RegExp, text: string, category: LinguisticCategory, out: LinguisticMatch[]) {
  re.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) {
    if (m[0].length === 0) {
      re.lastIndex++;
      continue;
    }
    out.push({ from: m.index, to: m.index + m[0].length, category });
  }
}

/** Find every linguistic pattern in `text`. Pure — no editor required. */
export function detectLinguistic(text: string, opts: LinguisticOptions): LinguisticMatch[] {
  if (!text) return [];
  const out: LinguisticMatch[] = [];
  if (opts.categories.includes('adverb')) {
    pushAll(ADVERB_RE, text, 'adverb', out);
    for (const raw of opts.extraWords) {
      const word = raw.trim();
      if (!word) continue;
      const re = new RegExp(`\\b${word.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\b`, 'giu');
      pushAll(re, text, 'adverb', out);
    }
  }
  if (opts.categories.includes('passive')) pushAll(PASSIVE_RE, text, 'passive', out);
  if (opts.categories.includes('dialogue')) pushAll(DIALOGUE_RE, text, 'dialogue', out);
  return out;
}

export const linguisticFocusKey = new PluginKey<LinguisticFocusPluginState>('linguisticFocus');

interface LinguisticFocusPluginState {
  enabled: boolean;
  options: LinguisticOptions;
}

const CLASS_BY_CATEGORY: Record<LinguisticCategory, string> = {
  adverb: 'lf-adverb',
  passive: 'lf-passive',
  dialogue: 'lf-dialogue',
};

function buildDecorations(doc: ProseMirrorNode, options: LinguisticOptions): DecorationSet {
  const decos: Decoration[] = [];
  doc.descendants((node, pos) => {
    if (!node.isText || !node.text) return;
    for (const match of detectLinguistic(node.text, options)) {
      decos.push(
        Decoration.inline(pos + match.from, pos + match.to, {
          class: CLASS_BY_CATEGORY[match.category],
        }),
      );
    }
  });
  return DecorationSet.create(doc, decos);
}

const DEFAULT_OPTIONS: LinguisticOptions = {
  categories: ['adverb', 'passive', 'dialogue'],
  extraWords: [],
};

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    linguisticFocus: {
      /** Enable/disable the overlay and (optionally) update its options. */
      setLinguisticFocus: (enabled: boolean, options?: Partial<LinguisticOptions>) => ReturnType;
    };
  }
}

export const LinguisticFocus = Extension.create({
  name: 'linguisticFocus',

  addCommands() {
    return {
      setLinguisticFocus:
        (enabled, options) =>
        ({ state, dispatch }) => {
          if (dispatch) {
            const prev = linguisticFocusKey.getState(state);
            const merged: LinguisticOptions = { ...DEFAULT_OPTIONS, ...prev?.options, ...options };
            dispatch(state.tr.setMeta(linguisticFocusKey, { enabled, options: merged }));
          }
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin<LinguisticFocusPluginState>({
        key: linguisticFocusKey,
        state: {
          init: () => ({ enabled: false, options: DEFAULT_OPTIONS }),
          apply(tr, value) {
            const meta = tr.getMeta(linguisticFocusKey) as LinguisticFocusPluginState | undefined;
            return meta ?? value;
          },
        },
        props: {
          decorations(state) {
            const pluginState = linguisticFocusKey.getState(state);
            if (!pluginState?.enabled) return null;
            return buildDecorations(state.doc, pluginState.options);
          },
        },
      }),
    ];
  },
});
