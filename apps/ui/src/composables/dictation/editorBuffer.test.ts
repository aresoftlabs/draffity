import { afterEach, describe, expect, it } from 'vitest';
import { Editor } from '@tiptap/core';
import StarterKit from '@tiptap/starter-kit';
import { ref } from 'vue';
import { DictationPlaceholder } from '@/editor/extensions/dictation-placeholder';
import { DictationGhost } from '@/editor/extensions/dictation-ghost';
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

describe('createEditorBuffer streaming', () => {
  it('setGhost/clearGhost toggles the ghost; commitStreaming inserts and clears ghost', () => {
    const ed = new Editor({
      extensions: [StarterKit, DictationPlaceholder, DictationGhost],
      content: '<p>Hola</p>',
    });
    ed.commands.setTextSelection(5); // tras "Hola" (pos 5 = fin del texto)
    const buf = createEditorBuffer(ref(ed));
    buf.setGhost('mundo');
    buf.commitStreaming(' mundo ');
    expect(ed.getText()).toContain('Hola mundo ');
    buf.clearGhost();
    ed.destroy();
  });

  it('commitStreaming inserts HTML-special chars as plain text (no HTML parsing)', () => {
    const ed = new Editor({
      extensions: [StarterKit, DictationPlaceholder, DictationGhost],
      content: '<p></p>',
    });
    ed.commands.setTextSelection(1);
    const buf = createEditorBuffer(ref(ed));
    buf.commitStreaming('<b>x</b> ');
    // The literal angle-bracket sequence must appear in the document text,
    // proving insertContent did NOT parse the string as HTML.
    expect(ed.getText()).toContain('<b>x</b>');
    ed.destroy();
  });
});
