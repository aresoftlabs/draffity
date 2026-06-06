import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';

/**
 * Texto "fantasma" del dictado en vivo (Fase 3): muestra la hipótesis parcial
 * (gris/itálica) en la posición del cursor, sin mutar el documento. Separado del
 * placeholder manual para no tocar ese flujo. El texto definitivo se inserta con
 * `insertContent` normal y luego se limpia el fantasma.
 */
interface GhostState {
  text: string | null;
}

export const dictationGhostKey = new PluginKey<GhostState>('dictationGhost');

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    dictationGhost: {
      setDictationGhost: (text: string) => ReturnType;
      clearDictationGhost: () => ReturnType;
    };
  }
}

export const DictationGhost = Extension.create({
  name: 'dictationGhost',

  addCommands() {
    return {
      setDictationGhost:
        (text: string) =>
        ({ state, dispatch }) => {
          if (dispatch) dispatch(state.tr.setMeta(dictationGhostKey, { type: 'set', text }));
          return true;
        },
      clearDictationGhost:
        () =>
        ({ state, dispatch }) => {
          if (dispatch) dispatch(state.tr.setMeta(dictationGhostKey, { type: 'clear' }));
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin<GhostState>({
        key: dictationGhostKey,
        state: {
          init: () => ({ text: null }),
          apply(tr, value) {
            const meta = tr.getMeta(dictationGhostKey) as
              | { type: 'set'; text: string }
              | { type: 'clear' }
              | undefined;
            if (meta?.type === 'set') return { text: meta.text };
            if (meta?.type === 'clear') return { text: null };
            return value;
          },
        },
        props: {
          decorations(state) {
            const s = dictationGhostKey.getState(state);
            if (!s?.text) return null;
            const el = document.createElement('span');
            el.className = 'dictation-ghost text-stone-400 italic opacity-70';
            el.textContent = s.text;
            return DecorationSet.create(state.doc, [
              Decoration.widget(state.selection.head, el, { side: 1 }),
            ]);
          },
        },
      }),
    ];
  },
});
