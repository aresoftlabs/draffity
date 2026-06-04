import { afterEach, describe, expect, it } from 'vitest';
import { Editor } from '@tiptap/core';
import StarterKit from '@tiptap/starter-kit';
import { DictationPlaceholder } from './dictation-placeholder';

function makeEditor(content: string) {
  return new Editor({ extensions: [StarterKit, DictationPlaceholder], content });
}

let editor: Editor;
afterEach(() => editor?.destroy());

describe('DictationPlaceholder', () => {
  it('inserts dictated text at the placeholder, mapped past edits made elsewhere', () => {
    editor = makeEditor('<p>Hola</p>');
    editor.commands.setTextSelection(5);
    editor.commands.addDictationPlaceholder();
    editor.commands.insertContentAt(1, 'X');
    editor.commands.replaceDictationPlaceholder(' mundo');
    expect(editor.getText()).toBe('XHola mundo');
  });

  it('replaceDictationPlaceholder returns false when there is no placeholder', () => {
    editor = makeEditor('<p>Hola</p>');
    expect(editor.commands.replaceDictationPlaceholder('x')).toBe(false);
  });

  it('clearDictationPlaceholder removes the pending marker', () => {
    editor = makeEditor('<p>Hola</p>');
    editor.commands.setTextSelection(5);
    editor.commands.addDictationPlaceholder();
    editor.commands.clearDictationPlaceholder();
    expect(editor.commands.replaceDictationPlaceholder(' mundo')).toBe(false);
  });
});
