import { Node, mergeAttributes } from '@tiptap/core';
import { VueNodeViewRenderer } from '@tiptap/vue-3';
import ImageNodeView from './image-node-view.vue';

/**
 * Inline-block image referencing a `MediaAsset` by id. The persisted HTML
 * is `<img data-media-id="<id>" alt="â€¦">` â€” never carries a `src` (the
 * Blob URL is local to the running session). At render time the
 * `ImageNodeView` Vue component pulls a Blob URL from the media store and
 * binds it; exporters resolve `data-media-id` to bytes themselves.
 */
export interface ImageAttributes {
  mediaId: string;
  alt: string;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    draffityImage: {
      insertImage: (attrs: ImageAttributes) => ReturnType;
    };
  }
}

export const Image = Node.create({
  name: 'image',
  group: 'block',
  atom: true,
  draggable: true,
  selectable: true,

  addAttributes() {
    return {
      mediaId: {
        default: '',
        parseHTML: (el) => el.getAttribute('data-media-id') ?? '',
        renderHTML: (attrs) => ({
          'data-media-id': (attrs as ImageAttributes).mediaId,
        }),
      },
      alt: {
        default: '',
        parseHTML: (el) => el.getAttribute('alt') ?? '',
        renderHTML: (attrs) => {
          const alt = (attrs as ImageAttributes).alt;
          return alt ? { alt } : {};
        },
      },
    };
  },

  parseHTML() {
    return [{ tag: 'img[data-media-id]' }];
  },

  renderHTML({ HTMLAttributes }) {
    // Persisted form never includes a `src` â€” bytes are local to the
    // session and re-resolved at load time by the NodeView.
    return ['img', mergeAttributes(HTMLAttributes, { class: 'draffity-image' })];
  },

  addCommands() {
    return {
      insertImage:
        (attrs) =>
        ({ chain }) => {
          return chain().focus().insertContent({ type: this.name, attrs }).run();
        },
    };
  },

  addNodeView() {
    return VueNodeViewRenderer(ImageNodeView);
  },
});
