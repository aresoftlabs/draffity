import { Node, mergeAttributes } from '@tiptap/core';
import { Plugin } from '@tiptap/pm/state';

/**
 * Inline cross-reference to a codex entry. Stored as
 * `<span data-codex-ref="<id>">[[Name]]</span>` â€” the inner text travels
 * through every exporter (md/docx/epub) untouched, and the `data-codex-ref`
 * attribute lets the runtime resolve clicks back to the entry id. Renames
 * don't break refs because the id is the source of truth; the visible name
 * is refreshed by the editor when bibliography labels are recomputed.
 *
 * Clicks bubble out via a `CustomEvent('draffity:open-codex')` on
 * `window` so callers can wire view-state changes without prop drilling
 * through TipTapEditor.
 */
export interface CodexRefAttributes {
  entryId: string;
  entryName: string;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    codexRef: {
      insertCodexRef: (attrs: CodexRefAttributes) => ReturnType;
    };
  }
}

export const CODEX_REF_EVENT = 'draffity:open-codex';

export interface CodexRefOpenDetail {
  id: string;
}

export const CodexRef = Node.create({
  name: 'codexRef',
  group: 'inline',
  inline: true,
  atom: true,
  selectable: true,

  addAttributes() {
    return {
      entryId: {
        default: '',
        parseHTML: (el) => el.getAttribute('data-codex-ref') ?? '',
        renderHTML: (attrs) => ({
          'data-codex-ref': (attrs as CodexRefAttributes).entryId,
        }),
      },
      entryName: {
        default: '',
        parseHTML: (el) => {
          const text = el.textContent ?? '';
          // Strip the `[[ ]]` wrapper when round-tripping HTML.
          return text.replace(/^\[\[/, '').replace(/\]\]$/, '');
        },
        renderHTML: () => ({}),
      },
    };
  },

  parseHTML() {
    return [{ tag: 'span[data-codex-ref]' }];
  },

  renderHTML({ node, HTMLAttributes }) {
    const name = (node.attrs as CodexRefAttributes).entryName || '?';
    return ['span', mergeAttributes(HTMLAttributes, { class: 'codex-ref' }), `[[${name}]]`];
  },

  renderText({ node }) {
    return `[[${(node.attrs as CodexRefAttributes).entryName}]]`;
  },

  addCommands() {
    return {
      insertCodexRef:
        (attrs) =>
        ({ chain }) => {
          return chain().focus().insertContent({ type: this.name, attrs }).insertContent(' ').run();
        },
    };
  },

  addProseMirrorPlugins() {
    // Single-pass click handler instead of a per-node DOM listener: cheaper
    // and survives editor re-renders without leaking.
    return [
      new Plugin({
        props: {
          handleClickOn(_view, _pos, node, _nodePos, event) {
            if (node.type.name !== 'codexRef') return false;
            const id = (node.attrs as CodexRefAttributes).entryId;
            if (!id) return false;
            event.preventDefault();
            window.dispatchEvent(
              new CustomEvent<CodexRefOpenDetail>(CODEX_REF_EVENT, { detail: { id } }),
            );
            return true;
          },
        },
      }),
    ];
  },
});
