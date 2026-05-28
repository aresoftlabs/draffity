import { onBeforeUnmount, watch, type Ref } from 'vue';
import type { Editor } from '@tiptap/vue-3';

/**
 * Keeps the current ProseMirror cursor line vertically centred inside the
 * editor's scroll container while `enabled` is true ("typewriter scroll").
 *
 * Reacts to selection changes and content updates; when disabled, leaves the
 * editor alone. Attaches/detaches when the editor instance becomes
 * available or changes.
 */
export function useTypewriterScroll(editorRef: Ref<Editor | null>, enabled: Ref<boolean>) {
  let detach: (() => void) | null = null;

  function recenter(ed: Editor) {
    if (!enabled.value) return;
    const { state, view } = ed;
    const pos = state.selection.head;
    let coords: { top: number; bottom: number; left: number; right: number };
    try {
      coords = view.coordsAtPos(pos);
    } catch {
      return; // pos may be out-of-bounds during transitions
    }
    // The scroll container is the nearest ancestor with overflow auto/scroll.
    const container = findScrollParent(view.dom);
    if (!container) return;
    const containerRect = container.getBoundingClientRect();
    const cursorYInContainer = coords.top - containerRect.top + container.scrollTop;
    const target = cursorYInContainer - container.clientHeight / 2;
    container.scrollTo({ top: Math.max(0, target), behavior: 'smooth' });
  }

  function attach(ed: Editor) {
    const handler = () => recenter(ed);
    ed.on('selectionUpdate', handler);
    ed.on('update', handler);
    detach = () => {
      ed.off('selectionUpdate', handler);
      ed.off('update', handler);
    };
  }

  watch(
    editorRef,
    (ed) => {
      detach?.();
      detach = null;
      if (ed) attach(ed);
    },
    { immediate: true },
  );

  // When enabled flips on, immediately centre the current line.
  watch(enabled, (on) => {
    if (on && editorRef.value) recenter(editorRef.value);
  });

  onBeforeUnmount(() => {
    detach?.();
    detach = null;
  });
}

function findScrollParent(el: Element | null): HTMLElement | null {
  let cursor: Element | null = el;
  while (cursor && cursor !== document.body) {
    const parent = cursor.parentElement;
    if (!parent) return null;
    const style = getComputedStyle(parent);
    const overflowY = style.overflowY;
    if (overflowY === 'auto' || overflowY === 'scroll') return parent;
    cursor = parent;
  }
  return null;
}
