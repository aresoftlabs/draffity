import { describe, expect, it, vi } from 'vitest';
import { defineComponent, h } from 'vue';
import { mount } from '@vue/test-utils';
import { useEditorAutoSave, type EditorDoc } from './useEditorAutoSave';

type Options = Parameters<typeof useEditorAutoSave>[0];

function host(options: Options) {
  type Editor = ReturnType<typeof useEditorAutoSave>;
  const captured: { editor: Editor | null } = { editor: null };
  const Comp = defineComponent({
    setup() {
      captured.editor = useEditorAutoSave(options);
      return () => h('div');
    },
  });
  const wrapper = mount(Comp);
  return {
    wrapper,
    get editor(): Editor {
      if (!captured.editor) throw new Error('editor not initialized');
      return captured.editor;
    },
  };
}

const docA: EditorDoc = { id: 'A', content: 'a-original', contentJson: '{"a":1}' };
const docB: EditorDoc = { id: 'B', content: 'b-original', contentJson: '{"b":2}' };

describe('useEditorAutoSave', () => {
  it('loads a document into the editor and binds to its id', async () => {
    const { editor } = host({ persist: vi.fn() });

    await editor.load(docA);

    expect(editor.content.value).toBe('a-original');
    expect(editor.contentJson.value).toBe('{"a":1}');
    expect(editor.boundId.value).toBe('A');
  });

  // AUD-01 / AUD-02 regression: switching the selected document must flush the
  // pending save against the document that is actually loaded in the editor,
  // never against the document that is about to be loaded.
  it('flushes the pending edit to the document being edited, not the next one', async () => {
    const saved: Array<{ id: string; content: string }> = [];
    const { editor } = host({
      persist: (id, payload) => {
        saved.push({ id, content: payload.content });
      },
    });

    await editor.load(docA);
    editor.onContent('a-edited'); // user edits A
    expect(editor.pending()).toBe(true);

    await editor.load(docB); // user switches selection to B before debounce fires

    expect(saved).toEqual([{ id: 'A', content: 'a-edited' }]);
    // The editor now shows B's content, bound to B.
    expect(editor.content.value).toBe('b-original');
    expect(editor.boundId.value).toBe('B');
    // No pending save leaked into B.
    expect(editor.pending()).toBe(false);
  });

  it('does not persist edits while read-only', async () => {
    const persist = vi.fn();
    const readOnly = { value: true };
    const { editor } = host({ persist, readOnly: () => readOnly.value });

    await editor.load(docA);
    editor.onContent('a-edited');

    await editor.flush();
    expect(persist).not.toHaveBeenCalled();
  });

  it('debounced edit persists to the bound document', async () => {
    vi.useFakeTimers();
    try {
      const persist = vi.fn();
      const { editor } = host({ persist, delay: 200 });

      await editor.load(docA);
      editor.onContent('a-edited');
      expect(persist).not.toHaveBeenCalled();

      vi.advanceTimersByTime(200);
      await Promise.resolve();

      expect(persist).toHaveBeenCalledWith('A', {
        content: 'a-edited',
        contentJson: '{"a":1}',
      });
    } finally {
      vi.useRealTimers();
    }
  });

  it('clears the editor when loading a null document', async () => {
    const { editor } = host({ persist: vi.fn() });

    await editor.load(docA);
    await editor.load(null);

    expect(editor.content.value).toBe('');
    expect(editor.contentJson.value).toBe(null);
    expect(editor.boundId.value).toBe(null);
  });
});
