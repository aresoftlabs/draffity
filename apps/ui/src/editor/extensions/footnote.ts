import { Node, mergeAttributes } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';

/**
 * Inline footnote node. The body text lives inline as a node attribute
 * (`data-footnote-content`) so the serialised HTML is self-contained â€” no
 * separate footnotes table to keep in sync. The visible glyph in the
 * editor is a dagger; numbering happens at export time based on document
 * order. Clicking a footnote dispatches `draffity:open-footnote` on
 * `window` carrying `{ id, content }` so a host dialog can handle the
 * edit flow without coupling the extension to PrimeVue.
 */
export interface FootnoteAttributes {
  id: string;
  content: string;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    footnote: {
      insertFootnote: (attrs: FootnoteAttributes) => ReturnType;
      updateFootnote: (id: string, content: string) => ReturnType;
    };
  }
}

export const Footnote = Node.create({
  name: 'footnote',
  group: 'inline',
  inline: true,
  atom: true,
  selectable: true,

  addAttributes() {
    return {
      id: {
        default: '',
        parseHTML: (el) => el.getAttribute('data-footnote-id') ?? '',
        renderHTML: (attrs) => ({
          'data-footnote-id': (attrs as FootnoteAttributes).id,
        }),
      },
      content: {
        default: '',
        parseHTML: (el) => el.getAttribute('data-footnote-content') ?? '',
        renderHTML: (attrs) => ({
          'data-footnote-content': (attrs as FootnoteAttributes).content,
        }),
      },
    };
  },

  parseHTML() {
    return [{ tag: 'sup[data-footnote-id]' }];
  },

  renderHTML({ HTMLAttributes }) {
    return ['sup', mergeAttributes(HTMLAttributes, { class: 'footnote-ref' }), 'â€ '];
  },

  renderText() {
    return 'â€ ';
  },

  addCommands() {
    return {
      insertFootnote:
        (attrs) =>
        ({ chain }) =>
          chain().focus().insertContent({ type: this.name, attrs }).run(),
      updateFootnote:
        (id, content) =>
        ({ tr, state, dispatch }) => {
          let changed = false;
          state.doc.descendants((node, pos) => {
            if (node.type.name !== 'footnote') return;
            if ((node.attrs as FootnoteAttributes).id !== id) return;
            tr.setNodeMarkup(pos, undefined, { ...node.attrs, content });
            changed = true;
          });
          if (changed && dispatch) dispatch(tr);
          return changed;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey('footnote-click'),
        props: {
          handleClickOn(_view, _pos, node) {
            if (node.type.name !== 'footnote') return false;
            const attrs = node.attrs as FootnoteAttributes;
            window.dispatchEvent(
              new CustomEvent('draffity:open-footnote', {
                detail: { id: attrs.id, content: attrs.content },
              }),
            );
            return true;
          },
        },
      }),
    ];
  },
});
