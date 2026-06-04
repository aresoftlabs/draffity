import { onUnmounted, ref, watch, type Ref } from 'vue';
import type { Editor } from '@tiptap/vue-3';
import { useVoiceRecorder } from '@/audio/useVoiceRecorder';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';
import { shouldConfirmDiscard } from '@/composables/dictationDiscard';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { ipc } from '@/services/ipc';
import type { VoiceTranscribeProgress } from '@/services/ipc';
import '@/editor/extensions/dictation-placeholder';

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

/** Resolve the preferred microphone `deviceId` from settings (null = default). */
export function resolveInputDeviceId(): string | null {
  try {
    const store = useVoiceSettingsStore();
    return store.inputDeviceId;
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
  /** Se llamó al fallback de portapapeles: el texto ya está copiado. */
  onClipboardFallback?: (text: string) => void;
  /** Confirmación de descarte para grabaciones largas. Default: confirma siempre que sí. */
  confirmDiscard?: () => boolean;
}

export function useDictation(editor: Ref<Editor | null>, options: DictationOptions = {}) {
  const recorder = useVoiceRecorder();
  const phase = ref<DictationPhase>('idle');
  const error = ref<string | null>(null);
  const progress = ref<number | null>(null);

  let runId = 0;

  function resolveAutoStop(): boolean {
    try {
      return useVoiceSettingsStore().autoStopOnSilence;
    } catch {
      return false;
    }
  }

  async function clipboardFallback(text: string) {
    try {
      await navigator.clipboard?.writeText(text);
    } catch {
      /* sin portapapeles: el onError de abajo igual avisa */
    }
    options.onClipboardFallback?.(text);
  }

  let unlistenProgress: UnlistenFn | null = null;
  let disposed = false;
  void listen<VoiceTranscribeProgress>('voice.transcribe.progress', (e) => {
    if (phase.value === 'transcribing') progress.value = e.payload.progress;
  }).then((un) => {
    if (disposed) un();
    else unlistenProgress = un;
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && phase.value === 'recording') {
      e.preventDefault();
      cancel();
    }
  }
  window.addEventListener('keydown', onKeydown);

  onUnmounted(() => {
    disposed = true;
    unlistenProgress?.();
    window.removeEventListener('keydown', onKeydown);
  });

  function fail(e: unknown) {
    const message = String((e as { message?: string })?.message ?? e);
    error.value = message;
    options.onError?.(message);
  }

  async function start() {
    error.value = null;
    try {
      await recorder.start(resolveInputDeviceId());
      phase.value = 'recording';
    } catch (e) {
      fail(e);
      phase.value = 'idle';
    }
  }

  async function stopAndInsert() {
    if (phase.value !== 'recording') return;
    const myRun = ++runId;
    editor.value?.commands.addDictationPlaceholder();
    progress.value = 0;
    phase.value = 'transcribing';
    try {
      const rec = await recorder.stop();
      const transcript = await ipc.transcribeAudio(rec.wav);
      if (myRun !== runId) return; // cancelado mientras transcribía
      const clean = transcript.text.trim();
      if (!clean) {
        editor.value?.commands.clearDictationPlaceholder();
      } else {
        const ok = editor.value?.commands.replaceDictationPlaceholder(`${clean} `) ?? false;
        if (!ok) await clipboardFallback(clean);
      }
    } catch (e) {
      if (myRun === runId) {
        editor.value?.commands.clearDictationPlaceholder();
        fail(e);
      }
    } finally {
      if (myRun === runId) {
        phase.value = 'idle';
        progress.value = null;
      }
    }
  }

  function cancel() {
    if (
      phase.value === 'recording' &&
      shouldConfirmDiscard(recorder.elapsedMs.value) &&
      !(options.confirmDiscard?.() ?? true)
    ) {
      return;
    }
    runId++; // invalida cualquier transcripción en vuelo
    recorder.cancel();
    editor.value?.commands.clearDictationPlaceholder?.();
    phase.value = 'idle';
    progress.value = null;
  }

  /** Toggle: start when idle, finish+insert when recording. */
  function toggle() {
    if (phase.value === 'recording') void stopAndInsert();
    else if (phase.value === 'idle') void start();
  }

  watch(
    () => recorder.isSilent.value,
    (silent) => {
      if (silent && phase.value === 'recording' && resolveAutoStop()) void stopAndInsert();
    },
  );

  return {
    phase,
    level: recorder.level,
    waveform: recorder.waveform,
    elapsedMs: recorder.elapsedMs,
    isSilent: recorder.isSilent,
    progress,
    error,
    start,
    stopAndInsert,
    cancel,
    toggle,
  };
}
