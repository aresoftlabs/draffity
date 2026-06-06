import { computed, onUnmounted, ref, watch, type Ref } from 'vue';
import type { Editor } from '@tiptap/vue-3';
import { useVoiceRecorder } from '@/audio/useVoiceRecorder';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { VoiceTranscribeProgress } from '@/services/ipc';
import { createEditorBuffer } from './dictation/editorBuffer';
import { createManualDictationMode } from './dictation/ManualDictationMode';
import { createStreamingDictationMode } from './dictation/StreamingDictationMode';
import { resolveAutoStop, resolveDictationMode } from './dictation/settings';
import type {
  DictationContext,
  DictationMode,
  DictationOptions,
  DictationPhase,
} from './dictation/types';

// Re-exports para compatibilidad con consumidores y tests existentes.
export { resolveAsrModelId, resolveInputDeviceId } from './dictation/settings';
export type { DictationPhase, DictationOptions } from './dictation/types';

/**
 * Host de dictado. Arma el contexto compartido (recorder + buffer de editor +
 * setters de fase/progreso) e instancia el modo activo (hoy: manual). Atajos,
 * auto-stop por silencio y el listener de progreso son concerns del host que
 * delegan en el modo. La superficie pública no cambia (cero regresión).
 */
export function useDictation(editor: Ref<Editor | null>, options: DictationOptions = {}) {
  const recorder = useVoiceRecorder();
  const phase = ref<DictationPhase>('idle');
  const error = ref<string | null>(null);
  const progress = ref<number | null>(null);

  async function clipboardFallback(text: string) {
    try {
      await navigator.clipboard?.writeText(text);
    } catch {
      /* sin portapapeles: el onError igual avisa */
    }
    options.onClipboardFallback?.(text);
  }

  function fail(e: unknown) {
    const message = String((e as { message?: string })?.message ?? e);
    error.value = message;
    options.onError?.(message);
  }

  const ctx: DictationContext = {
    editor: createEditorBuffer(editor),
    recorder,
    options,
    setPhase: (p) => (phase.value = p),
    setProgress: (v) => (progress.value = v),
    fail,
    clipboardFallback,
  };

  let activeMode: DictationMode = createManualDictationMode();
  const activeModeId = ref<'manual' | 'streaming'>('manual');
  function pickMode(): DictationMode {
    return resolveDictationMode() === 'streaming'
      ? createStreamingDictationMode()
      : createManualDictationMode();
  }

  let unlistenProgress: UnlistenFn | null = null;
  let disposed = false;
  void listen<VoiceTranscribeProgress>('voice:transcribe:progress', (e) => {
    if (phase.value === 'transcribing') progress.value = e.payload.progress;
  }).then((un) => {
    if (disposed) un();
    else unlistenProgress = un;
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && phase.value === 'recording') {
      e.preventDefault();
      activeMode.cancel(ctx);
    }
  }
  window.addEventListener('keydown', onKeydown);

  onUnmounted(() => {
    disposed = true;
    unlistenProgress?.();
    window.removeEventListener('keydown', onKeydown);
  });

  function start() {
    error.value = null;
    activeMode = pickMode();
    activeModeId.value = activeMode.id;
    void activeMode.start(ctx);
  }
  function stopAndInsert() {
    if (phase.value === 'recording') void activeMode.stop(ctx);
  }
  function cancel() {
    activeMode.cancel(ctx);
  }
  function toggle() {
    if (phase.value === 'recording') stopAndInsert();
    else if (phase.value === 'idle') start();
  }

  watch(
    () => recorder.isSilent.value,
    (silent) => {
      if (silent && phase.value === 'recording' && resolveAutoStop()) void activeMode.stop(ctx);
    },
  );

  type CaptureHandle = {
    capture?: {
      level: { value: number };
      waveform: { value: Uint8Array };
      elapsedMs: { value: number };
      isSilent: { value: boolean };
    };
  };
  const streamingCapture = () => (activeMode as DictationMode & CaptureHandle).capture;
  const level = computed(() =>
    activeModeId.value === 'streaming'
      ? (streamingCapture()?.level.value ?? 0)
      : recorder.level.value,
  );
  const waveform = computed(() =>
    activeModeId.value === 'streaming'
      ? (streamingCapture()?.waveform.value ?? new Uint8Array(0))
      : recorder.waveform.value,
  );
  const elapsedMs = computed(() =>
    activeModeId.value === 'streaming'
      ? (streamingCapture()?.elapsedMs.value ?? 0)
      : recorder.elapsedMs.value,
  );
  const isSilent = computed(() =>
    activeModeId.value === 'streaming' ? false : recorder.isSilent.value,
  );

  return {
    phase,
    level,
    waveform,
    elapsedMs,
    isSilent,
    progress,
    error,
    start,
    stopAndInsert,
    cancel,
    toggle,
  };
}
