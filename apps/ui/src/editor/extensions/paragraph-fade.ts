import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';

/**
 * Paragraph fade (K-08, composition mode): dims every top-level block except
 * the one holding the cursor, so the writer's eye rests on the current
 * paragraph. Pure decorations — never mutates the document. Toggled via
 * `setParagraphFade`; recomputes on every selection/doc change.
 */

export const paragraphFadeKey = new PluginKey<{ enabled: boolean }>('paragraphFade');

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    paragraphFade: {
      setParagraphFade: (enabled: boolean) => ReturnType;
    };
  }
}

export const ParagraphFade = Extension.create({
  name: 'paragraphFade',

  addCommands() {
    return {
      setParagraphFade:
        (enabled) =>
        ({ state, dispatch }) => {
          if (dispatch) dispatch(state.tr.setMeta(paragraphFadeKey, { enabled }));
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin<{ enabled: boolean }>({
        key: paragraphFadeKey,
        state: {
          init: () => ({ enabled: false }),
          apply: (tr, value) => tr.getMeta(paragraphFadeKey) ?? value,
        },
        props: {
          decorations(state) {
            const s = paragraphFadeKey.getState(state);
            if (!s?.enabled) return null;
            const head = state.selection.head;
            const decos: Decoration[] = [];
            // Walk top-level blocks; dim every one that doesn't contain the cursor.
            state.doc.forEach((node, offset) => {
              const start = offset;
              const end = offset + node.nodeSize;
              const focused = head >= start && head <= end;
              if (!focused) {
                decos.push(Decoration.node(start, end, { class: 'pm-faded' }));
              }
            });
            return DecorationSet.create(state.doc, decos);
          },
        },
      }),
    ];
  },
});
