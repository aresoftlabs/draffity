import { ref, type Ref } from 'vue';
import type { Editor } from '@tiptap/vue-3';
import { useAudioRecorder } from '@/audio/useAudioRecorder';
import { ipc } from '@/services/ipc';

/**
 * Dictation orchestration (H-04). Wires the (engine-agnostic) recorder to the
 * (engine-agnostic) `transcribe_audio` command and inserts the result at the
 * editor cursor. Knows nothing about Whisper specifically — the backend picks
 * the ASR engine, so swapping it is invisible here.
 */
export type DictationPhase = 'idle' | 'recording' | 'transcribing';

export function useDictation(editor: Ref<Editor | null>) {
  const recorder = useAudioRecorder();
  const phase = ref<DictationPhase>('idle');
  const error = ref<string | null>(null);

  async function start() {
    error.value = null;
    try {
      await recorder.start();
      phase.value = 'recording';
    } catch (e) {
      error.value = String((e as { message?: string })?.message ?? e);
      phase.value = 'idle';
    }
  }

  async function stopAndInsert() {
    if (phase.value !== 'recording') return;
    phase.value = 'transcribing';
    try {
      const rec = await recorder.stop();
      const transcript = await ipc.transcribeAudio(rec.wav);
      insertAtCursor(transcript.text);
    } catch (e) {
      error.value = String((e as { message?: string })?.message ?? e);
    } finally {
      phase.value = 'idle';
    }
  }

  function cancel() {
    recorder.cancel();
    phase.value = 'idle';
  }

  /** Toggle: start when idle, finish+insert when recording. */
  function toggle() {
    if (phase.value === 'recording') void stopAndInsert();
    else if (phase.value === 'idle') void start();
  }

  function insertAtCursor(text: string) {
    const ed = editor.value;
    const clean = text.trim();
    if (!ed || !clean) return;
    // Whisper (esp. turbo) already punctuates; insert as plain text + a
    // trailing space so the next dictation reads naturally.
    ed.chain().focus().insertContent(`${clean} `).run();
  }

  return {
    phase,
    level: recorder.level,
    error,
    start,
    stopAndInsert,
    cancel,
    toggle,
  };
}
