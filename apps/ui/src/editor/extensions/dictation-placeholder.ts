import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';

/**
 * Marcador de dictado (spec §7). Mientras se transcribe, guarda la posición
 * del cursor como una *widget decoration* "▍ transcribiendo…" que no muta el
 * documento y cuya posición se mapea a través de las transacciones siguientes,
 * para que el texto dictado caiga donde el usuario estaba aunque siga escribiendo.
 */

interface PlaceholderState {
  pos: number | null;
}

export const dictationPlaceholderKey = new PluginKey<PlaceholderState>('dictationPlaceholder');

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    dictationPlaceholder: {
      addDictationPlaceholder: () => ReturnType;
      replaceDictationPlaceholder: (text: string) => ReturnType;
      clearDictationPlaceholder: () => ReturnType;
    };
  }
}

export const DictationPlaceholder = Extension.create({
  name: 'dictationPlaceholder',

  addCommands() {
    return {
      addDictationPlaceholder:
        () =>
        ({ state, dispatch }) => {
          if (dispatch) {
            dispatch(
              state.tr.setMeta(dictationPlaceholderKey, { type: 'add', pos: state.selection.from }),
            );
          }
          return true;
        },
      clearDictationPlaceholder:
        () =>
        ({ state, dispatch }) => {
          if (dispatch) dispatch(state.tr.setMeta(dictationPlaceholderKey, { type: 'clear' }));
          return true;
        },
      replaceDictationPlaceholder:
        (text: string) =>
        ({ state, dispatch, tr }) => {
          const s = dictationPlaceholderKey.getState(state);
          const pos = s?.pos ?? null;
          if (pos == null) return false;
          if (dispatch) {
            tr.insertText(text, pos);
            tr.setMeta(dictationPlaceholderKey, { type: 'clear' });
            dispatch(tr);
          }
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin<PlaceholderState>({
        key: dictationPlaceholderKey,
        state: {
          init: () => ({ pos: null }),
          apply(tr, value) {
            const meta = tr.getMeta(dictationPlaceholderKey) as
              | { type: 'add'; pos: number }
              | { type: 'clear' }
              | undefined;
            if (meta?.type === 'add') return { pos: meta.pos };
            if (meta?.type === 'clear') return { pos: null };
            if (value.pos == null) return value;
            return { pos: tr.mapping.map(value.pos) };
          },
        },
        props: {
          decorations(state) {
            const s = dictationPlaceholderKey.getState(state);
            if (s?.pos == null) return null;
            const el = document.createElement('span');
            el.className = 'dictation-placeholder text-amber-500 opacity-70';
            el.textContent = '▍ transcribiendo…';
            return DecorationSet.create(state.doc, [Decoration.widget(s.pos, el, { side: 1 })]);
          },
        },
      }),
    ];
  },
});
