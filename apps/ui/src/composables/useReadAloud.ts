import { useVoiceSettingsStore } from '@/stores/voiceSettings';
import { onBeforeUnmount, ref, type Ref } from 'vue';
import type { Editor } from '@tiptap/vue-3';
import { ipc } from '@/services/ipc';
import { findMatches } from './useProseMirrorSearch';

/**
 * Resolve the current TTS voice ID from the settings store.
 * Falls back to empty string when none is set (legacy behavior).
 */
export function resolveVoiceId(): string {
  // Guard: Pinia store may not be active outside setup().
  try {
    const store = useVoiceSettingsStore();
    return store.ttsVoiceId ?? '';
  } catch {
    return '';
  }
}

/**
 * Read-aloud (H-07). Splits the document into sentences and synthesizes them
 * one at a time (so the current sentence can be highlighted), playing each via
 * the Web Audio API. Engine-agnostic: it only calls `synthesize_speech`, so the
 * TTS backend can be swapped with no change here. A `runId` guards against
 * stale async callbacks after stop/skip.
 */
export type ReadAloudPhase = 'idle' | 'playing' | 'paused';

const SPEEDS = [0.75, 1, 1.25, 1.5] as const;

export interface ReadAloudOptions {
  /** Called when synthesis fails so the host can surface it — the `error` ref
   *  alone was never rendered (AUD-15). */
  onError?: (message: string) => void;
}

export function useReadAloud(editor: Ref<Editor | null>, options: ReadAloudOptions = {}) {
  const phase = ref<ReadAloudPhase>('idle');
  const speed = ref(1);
  const error = ref<string | null>(null);

  let ctx: AudioContext | null = null;
  let source: AudioBufferSourceNode | null = null;
  let sentences: string[] = [];
  let index = 0;
  let runId = 0;

  function splitSentences(text: string): string[] {
    return text
      .split(/(?<=[.!?…])\s+|\n+/)
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  }

  function highlight(sentence: string) {
    const ed = editor.value;
    if (!ed) return;
    const matches = findMatches(ed.state.doc, sentence, false);
    if (matches.length > 0) {
      ed.chain().focus().setTextSelection(matches[0]).scrollIntoView().run();
    }
  }

  function stopSource() {
    if (source) {
      try {
        source.onended = null;
        source.stop();
      } catch {
        // already stopped
      }
      source = null;
    }
  }

  async function playCurrent(myRun: number) {
    if (myRun !== runId) return;
    if (index >= sentences.length) {
      stop();
      return;
    }
    const sentence = sentences[index];
    highlight(sentence);
    let audio;
    try {
      audio = await ipc.synthesizeSpeech(sentence, resolveVoiceId());
    } catch (e) {
      const message = String((e as { message?: string })?.message ?? e);
      error.value = message;
      options.onError?.(message);
      stop();
      return;
    }
    if (myRun !== runId) return;
    if (!ctx) ctx = new AudioContext();
    const len = Math.max(1, audio.samplesPcm16.length);
    const buffer = ctx.createBuffer(1, len, audio.sampleRate || 22050);
    const channel = buffer.getChannelData(0);
    for (let i = 0; i < audio.samplesPcm16.length; i++) channel[i] = audio.samplesPcm16[i] / 32768;
    source = ctx.createBufferSource();
    source.buffer = buffer;
    source.playbackRate.value = speed.value;
    source.connect(ctx.destination);
    source.onended = () => {
      if (myRun !== runId || phase.value !== 'playing') return;
      index += 1;
      void playCurrent(myRun);
    };
    source.start();
  }

  async function play() {
    const ed = editor.value;
    if (!ed) return;
    stop();
    error.value = null;
    sentences = splitSentences(ed.getText());
    if (sentences.length === 0) return;
    index = 0;
    runId += 1;
    phase.value = 'playing';
    void playCurrent(runId);
  }

  function pause() {
    if (phase.value !== 'playing' || !ctx) return;
    void ctx.suspend();
    phase.value = 'paused';
  }

  function resume() {
    if (phase.value !== 'paused' || !ctx) return;
    void ctx.resume();
    phase.value = 'playing';
  }

  function stop() {
    runId += 1;
    phase.value = 'idle';
    stopSource();
    if (ctx) {
      void ctx.close();
      ctx = null;
    }
  }

  function skip() {
    if (phase.value === 'idle') return;
    stopSource();
    index += 1;
    void playCurrent(runId);
  }

  function setSpeed(v: number) {
    speed.value = v;
    if (source) source.playbackRate.value = v;
  }

  function toggle() {
    if (phase.value === 'idle') void play();
    else if (phase.value === 'playing') pause();
    else resume();
  }

  onBeforeUnmount(stop);

  return { phase, speed, error, speeds: SPEEDS, play, pause, resume, stop, skip, setSpeed, toggle };
}
