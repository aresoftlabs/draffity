import { afterEach, describe, expect, it } from 'vitest';
import { Editor } from '@tiptap/core';
import StarterKit from '@tiptap/starter-kit';
import { ref } from 'vue';
import { DictationPlaceholder } from '@/editor/extensions/dictation-placeholder';
import { createEditorBuffer } from './editorBuffer';

function makeEditor(content: string) {
  return new Editor({ extensions: [StarterKit, DictationPlaceholder], content });
}

let editor: Editor;
afterEach(() => editor?.destroy());

describe('createEditorBuffer', () => {
  it('commits text at the pending anchor, mapped past edits elsewhere', () => {
    editor = makeEditor('<p>Hola</p>');
    editor.commands.setTextSelection(5);
    const buf = createEditorBuffer(ref(editor));
    buf.beginPending();
    editor.commands.insertContentAt(1, 'X');
    expect(buf.commit(' mundo')).toBe(true);
    expect(editor.getText()).toBe('XHola mundo');
  });

  it('commit returns false when there is no pending anchor', () => {
    editor = makeEditor('<p>Hola</p>');
    const buf = createEditorBuffer(ref(editor));
    expect(buf.commit('x')).toBe(false);
  });

  it('clearPending drops the anchor', () => {
    editor = makeEditor('<p>Hola</p>');
    editor.commands.setTextSelection(5);
    const buf = createEditorBuffer(ref(editor));
    buf.beginPending();
    buf.clearPending();
    expect(buf.commit(' mundo')).toBe(false);
  });

  it('is a no-op safe when the editor ref is null', () => {
    const buf = createEditorBuffer(ref(null));
    expect(() => buf.beginPending()).not.toThrow();
    expect(buf.commit('x')).toBe(false);
    expect(() => buf.clearPending()).not.toThrow();
  });
});
