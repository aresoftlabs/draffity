import { computed, ref, watch, type Ref } from 'vue';
import type { Editor } from '@tiptap/vue-3';
import { findMatches, type FindMatch } from './useProseMirrorSearch';

export type { FindMatch };

/**
 * In-document find & replace driven by ProseMirror positions. Walks the
 * editor's text nodes on demand (via `findMatches` in `useProseMirrorSearch`)
 * to locate matches, then drives the editor's selection to jump between
 * them. Replacement uses the editor's transaction chain so undo works
 * naturally.
 *
 * Caller passes a reactive ref to the live `Editor`; the composable
 * subscribes to its `update` event so external edits invalidate the cache.
 */
export function useFindReplace(editorRef: Ref<Editor | null>) {
  const query = ref('');
  const replacement = ref('');
  const caseSensitive = ref(false);
  const matches = ref<FindMatch[]>([]);
  const currentIndex = ref(-1);

  const totalMatches = computed(() => matches.value.length);
  const hasMatches = computed(() => matches.value.length > 0);

  function recompute(jumpToFirst = true) {
    const ed = editorRef.value;
    if (!ed) {
      matches.value = [];
      currentIndex.value = -1;
      return;
    }
    const found = findMatches(ed.state.doc, query.value, caseSensitive.value);
    matches.value = found;
    if (jumpToFirst) {
      currentIndex.value = found.length > 0 ? 0 : -1;
      if (currentIndex.value >= 0) selectCurrent();
    } else if (currentIndex.value >= found.length) {
      currentIndex.value = found.length - 1;
      if (currentIndex.value >= 0) selectCurrent();
    }
  }

  function selectCurrent() {
    const ed = editorRef.value;
    const m = matches.value[currentIndex.value];
    if (!ed || !m) return;
    ed.chain().focus().setTextSelection({ from: m.from, to: m.to }).scrollIntoView().run();
  }

  function next() {
    if (matches.value.length === 0) return;
    currentIndex.value = (currentIndex.value + 1) % matches.value.length;
    selectCurrent();
  }

  function prev() {
    if (matches.value.length === 0) return;
    currentIndex.value = (currentIndex.value - 1 + matches.value.length) % matches.value.length;
    selectCurrent();
  }

  /** Replace the current match with `replacement`; advance to the next match. */
  function replaceCurrent() {
    const ed = editorRef.value;
    const m = matches.value[currentIndex.value];
    if (!ed || !m) return;
    ed.chain()
      .focus()
      .setTextSelection({ from: m.from, to: m.to })
      .insertContent(replacement.value)
      .run();
    // After splice, positions for following matches shifted — recompute. We
    // try to keep the cursor near where we were.
    recompute(false);
  }

  /** Replace every match in document order, back to front to preserve offsets. */
  function replaceAll(): number {
    const ed = editorRef.value;
    if (!ed || matches.value.length === 0) return 0;
    const ordered = [...matches.value].sort((a, b) => b.from - a.from);
    const tr = ed.state.tr;
    for (const m of ordered) {
      tr.insertText(replacement.value, m.from, m.to);
    }
    ed.view.dispatch(tr);
    const count = ordered.length;
    recompute();
    return count;
  }

  // External edits invalidate our cached match list.
  watch(editorRef, (ed, _, onCleanup) => {
    if (!ed) return;
    const handler = () => {
      if (query.value) recompute(false);
    };
    ed.on('update', handler);
    onCleanup(() => ed.off('update', handler));
  });

  // Recompute on query / case-sensitivity changes.
  watch([query, caseSensitive], () => recompute());

  function reset() {
    query.value = '';
    replacement.value = '';
    matches.value = [];
    currentIndex.value = -1;
  }

  return {
    query,
    replacement,
    caseSensitive,
    matches,
    currentIndex,
    totalMatches,
    hasMatches,
    next,
    prev,
    replaceCurrent,
    replaceAll,
    reset,
  };
}
