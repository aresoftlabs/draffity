import { Node, mergeAttributes } from '@tiptap/core';

/**
 * Inline citation node. The label (`(Surname, Year)`) is pre-rendered at
 * insert time and stored as the node's text content. This keeps export
 * trivial — the HTML serialised by `getHTML()` already contains the
 * resolved label inside a `<span data-citation-key="…">…</span>`. If the
 * underlying bibliography entry changes later, the editor offers a
 * "refresh citations" pass that walks the doc and updates each label.
 *
 * The node is **atom-like**: TipTap renders the inner label but the user
 * can't put the cursor inside it. Selection treats the span as a single
 * unit.
 */
export interface CitationAttributes {
  citationKey: string;
  label: string;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    citation: {
      insertCitation: (attrs: CitationAttributes) => ReturnType;
    };
  }
}

export const Citation = Node.create({
  name: 'citation',
  group: 'inline',
  inline: true,
  atom: true,
  selectable: true,

  addAttributes() {
    return {
      citationKey: {
        default: '',
        parseHTML: (el) => el.getAttribute('data-citation-key') ?? '',
        renderHTML: (attrs) => ({
          'data-citation-key': (attrs as CitationAttributes).citationKey,
        }),
      },
      label: {
        default: '',
        parseHTML: (el) => el.textContent ?? '',
        // The label rides inside the element's text content, not as an
        // attribute — so no contribution to attribute serialisation.
        renderHTML: () => ({}),
      },
    };
  },

  parseHTML() {
    return [{ tag: 'span[data-citation-key]' }];
  },

  renderHTML({ node, HTMLAttributes }) {
    const label = (node.attrs as CitationAttributes).label;
    return ['span', mergeAttributes(HTMLAttributes, { class: 'citation' }), label];
  },

  renderText({ node }) {
    // Used by `getText()` and some plain-text serialisers.
    return (node.attrs as CitationAttributes).label;
  },

  addCommands() {
    return {
      insertCitation:
        (attrs) =>
        ({ chain }) => {
          return chain().focus().insertContent({ type: this.name, attrs }).insertContent(' ').run();
        },
    };
  },
});
