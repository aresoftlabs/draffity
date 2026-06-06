import { toRaw, type Ref } from 'vue';
import type { SingleCommands } from '@tiptap/core';
import '@/editor/extensions/dictation-placeholder';
import '@/editor/extensions/dictation-ghost';
import type { EditorDictationBuffer } from './types';

/** Interfaz mínima del Editor que este buffer necesita. */
interface EditorLike {
  commands: SingleCommands;
}

/**
 * Costura de inserción: envuelve los comandos del plugin de placeholder para
 * que los modos inserten texto sin conocer TipTap. En la Fase 3 esta interfaz
 * se extiende con `setGhost/clearGhost` para el texto provisional del streaming.
 *
 * Nota: se usa `toRaw` para obtener la instancia real del Editor antes de
 * llamar a los comandos de TipTap/ProseMirror, ya que el ref de Vue envuelve
 * el objeto en un Proxy reactivo que puede corromper el rastreo de documentos
 * internos de ProseMirror (tr.before vs state.doc).
 */
export function createEditorBuffer(editor: Ref<EditorLike | null>): EditorDictationBuffer {
  function rawEditor() {
    return toRaw(editor.value) as EditorLike | null | undefined;
  }

  return {
    beginPending() {
      rawEditor()?.commands.addDictationPlaceholder();
    },
    commit(text: string): boolean {
      return rawEditor()?.commands.replaceDictationPlaceholder(text) ?? false;
    },
    clearPending() {
      rawEditor()?.commands.clearDictationPlaceholder?.();
    },
    setGhost(text: string) {
      rawEditor()?.commands.setDictationGhost?.(text);
    },
    clearGhost() {
      rawEditor()?.commands.clearDictationGhost?.();
    },
    commitStreaming(text: string): boolean {
      const ed = rawEditor();
      if (!ed) return false;
      ed.commands.clearDictationGhost?.();
      return ed.commands.insertContent(text) ?? false;
    },
  };
}
