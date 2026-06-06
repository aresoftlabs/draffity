import { afterEach, describe, expect, it } from 'vitest';
import { Editor } from '@tiptap/core';
import StarterKit from '@tiptap/starter-kit';
import { DictationGhost, dictationGhostKey } from './dictation-ghost';

function makeEditor(content: string) {
  return new Editor({ extensions: [StarterKit, DictationGhost], content });
}
let editor: Editor;
afterEach(() => editor?.destroy());

describe('DictationGhost', () => {
  it('stores ghost text in plugin state and clears it', () => {
    editor = makeEditor('<p>Hola</p>');
    editor.commands.setDictationGhost('mundo');
    expect(dictationGhostKey.getState(editor.state)?.text).toBe('mundo');
    editor.commands.clearDictationGhost();
    expect(dictationGhostKey.getState(editor.state)?.text).toBeNull();
  });
});
