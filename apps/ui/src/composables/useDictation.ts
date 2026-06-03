import { ref, type Ref } from 'vue';
import type { Editor } from '@tiptap/vue-3';
import { useAudioRecorder } from '@/audio/useAudioRecorder';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';
import { ipc } from '@/services/ipc';

/**
 * Resolve the current ASR model ID from the settings store.
 * Returns null when none is set (backend uses its default).
 */
export function resolveAsrModelId(): string | null {
  try {
    const store = useVoiceSettingsStore();
    return store.asrModelId;
  } catch {
    return null;
  }
}

/**
 * Dictation orchestration (H-04). Wires the (engine-agnostic) recorder to the
 * (engine-agnostic) `transcribe_audio` command and inserts the result at the
 * editor cursor. Knows nothing about Whisper specifically — the backend picks
 * the ASR engine, so swapping it is invisible here.
 */
export type DictationPhase = 'idle' | 'recording' | 'transcribing';

export interface DictationOptions {
  /** Called whenever an error is surfaced (mic denied, ASR failure) so the
   *  host can show it — the `error` ref alone was never rendered (AUD-15). */
  onError?: (message: string) => void;
}

export function useDictation(editor: Ref<Editor | null>, options: DictationOptions = {}) {
  const recorder = useAudioRecorder();
  const phase = ref<DictationPhase>('idle');
  const error = ref<string | null>(null);

  function fail(e: unknown) {
    const message = String((e as { message?: string })?.message ?? e);
    error.value = message;
    options.onError?.(message);
  }

  async function start() {
    error.value = null;
    try {
      await recorder.start();
      phase.value = 'recording';
    } catch (e) {
      fail(e);
      phase.value = 'idle';
    }
  }

  async function stopAndInsert() {
    if (phase.value !== 'recording') return;
    phase.value = 'transcribing';
    try {
      const rec = await recorder.stop();
      const transcript = await ipc.transcribeAudio(rec.wav, resolveAsrModelId());
      insertAtCursor(transcript.text);
    } catch (e) {
      fail(e);
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
