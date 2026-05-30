import { ref, type Ref } from 'vue';
import { useAutoSave } from './useAutoSave';

/** Minimal shape of a document the editor can load. */
export interface EditorDoc {
  id: string;
  content?: string | null;
  contentJson?: string | null;
}

export interface EditorAutoSaveOptions {
  /** Persist the editor content for a given document id. The return value is
   *  awaited but otherwise ignored, so wrappers returning `Promise<T | null>`
   *  (e.g. an error-handling `run`) fit without coercion. */
  persist: (id: string, payload: { content: string; contentJson?: string }) => unknown;
  /** Debounce window in ms. */
  delay?: number;
  /** When it returns true, edits are neither triggered nor persisted. */
  readOnly?: () => boolean;
}

/**
 * Owns the editor content together with the id of the document it belongs to,
 * and autosaves edits back to *that* document.
 *
 * The id is bound atomically with the content in `load()`, and the save always
 * targets `boundId` — never an externally-held selection. This is what prevents
 * a fast document switch from flushing doc A's pending content under doc B's id
 * (AUD-01 / AUD-02): `load()` flushes the previous document before reassigning.
 */
export function useEditorAutoSave(options: EditorAutoSaveOptions) {
  const content: Ref<string> = ref('');
  const contentJson: Ref<string | null> = ref(null);
  const boundId: Ref<string | null> = ref(null);

  const auto = useAutoSave(async () => {
    const id = boundId.value;
    if (!id || options.readOnly?.()) return;
    await options.persist(id, {
      content: content.value,
      contentJson: contentJson.value ?? undefined,
    });
  }, options.delay ?? 500);

  /**
   * Flush any pending save for the currently-loaded document, then load `doc`.
   * Order matters: the flush captures (boundId, content) before they change.
   */
  async function load(doc: EditorDoc | null) {
    await auto.flush();
    content.value = doc?.content ?? '';
    contentJson.value = doc?.contentJson ?? null;
    boundId.value = doc?.id ?? null;
  }

  function onContent(value: string) {
    content.value = value;
    if (!options.readOnly?.()) auto.trigger();
  }

  function onContentJson(value: string) {
    contentJson.value = value;
  }

  return {
    content,
    contentJson,
    boundId,
    load,
    onContent,
    onContentJson,
    flush: auto.flush,
    pending: auto.pending,
  };
}
