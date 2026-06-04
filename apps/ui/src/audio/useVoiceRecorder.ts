import { onBeforeUnmount, ref } from 'vue';
import { encodeWav } from './wav';

/**
 * Microphone recorder, decoupled from any consumer. Captures via MediaRecorder,
 * exposes a live RMS `level` for a meter, and on `stop()` decodes + resamples
 * to **16 kHz mono WAV** (what the local ASR expects) — so swapping the speech
 * engine never touches capture. `cancel()` discards without producing audio.
 */
export type RecorderState = 'idle' | 'recording';

export interface Recording {
  wav: Uint8Array;
  durationMs: number;
}

const TARGET_RATE = 16000;

export function useVoiceRecorder() {
  const state = ref<RecorderState>('idle');
  /** RMS amplitude 0..1 for a level meter / waveform. */
  const level = ref(0);
  /** Datos time-domain (0..255, 128 = silencio) del analyser, para la onda. */
  const waveform = ref<Uint8Array>(new Uint8Array(0));
  /** Tiempo grabado en ms (para el temporizador). */
  const elapsedMs = ref(0);
  /** `true` tras ~2 s por debajo del umbral de RMS mientras se graba. */
  const isSilent = ref(false);

  let stream: MediaStream | null = null;
  let audioCtx: AudioContext | null = null;
  let analyser: AnalyserNode | null = null;
  let recorder: MediaRecorder | null = null;
  let chunks: BlobPart[] = [];
  let raf = 0;
  let startedAt = 0;
  let discarded = false;
  let resolveStop: ((r: Recording) => void) | null = null;
  let rejectStop: ((e: unknown) => void) | null = null;

  const SILENCE_RMS = 0.02;
  const SILENCE_MS = 2000;
  let silentSince = 0;

  function tick() {
    if (!analyser) return;
    const buf = new Uint8Array(analyser.fftSize);
    analyser.getByteTimeDomainData(buf);
    let sum = 0;
    for (const v of buf) {
      const c = (v - 128) / 128;
      sum += c * c;
    }
    const rms = Math.sqrt(sum / buf.length);
    level.value = rms;
    waveform.value = buf;
    elapsedMs.value = performance.now() - startedAt;
    if (rms < SILENCE_RMS) {
      if (silentSince === 0) silentSince = performance.now();
      else if (performance.now() - silentSince >= SILENCE_MS) isSilent.value = true;
    } else {
      silentSince = 0;
      isSilent.value = false;
    }
    raf = requestAnimationFrame(tick);
  }

  function cleanup() {
    cancelAnimationFrame(raf);
    stream?.getTracks().forEach((t) => t.stop());
    stream = null;
    analyser = null;
    recorder = null;
    if (audioCtx && audioCtx.state !== 'closed') void audioCtx.close();
    audioCtx = null;
    level.value = 0;
    waveform.value = new Uint8Array(0);
    elapsedMs.value = 0;
    isSilent.value = false;
    silentSince = 0;
    state.value = 'idle';
  }

  async function resampleToMono16k(decoded: AudioBuffer): Promise<Float32Array> {
    const frames = Math.max(1, Math.ceil(decoded.duration * TARGET_RATE));
    const offline = new OfflineAudioContext(1, frames, TARGET_RATE);
    const src = offline.createBufferSource();
    src.buffer = decoded;
    src.connect(offline.destination);
    src.start();
    const rendered = await offline.startRendering();
    return rendered.getChannelData(0);
  }

  async function finalize() {
    const durationMs = Math.round(performance.now() - startedAt);
    const type = (chunks[0] as Blob | undefined)?.type || 'audio/webm';
    const blob = new Blob(chunks, { type });
    chunks = [];
    if (discarded) {
      cleanup();
      return;
    }
    try {
      const arr = await blob.arrayBuffer();
      const ctx = audioCtx ?? new AudioContext();
      const decoded = await ctx.decodeAudioData(arr.slice(0));
      const samples = await resampleToMono16k(decoded);
      const wav = encodeWav(samples, TARGET_RATE);
      const resolve = resolveStop;
      cleanup();
      resolve?.({ wav, durationMs });
    } catch (e) {
      const reject = rejectStop;
      cleanup();
      reject?.(e);
    } finally {
      resolveStop = null;
      rejectStop = null;
    }
  }

  async function start(deviceId?: string | null) {
    if (state.value === 'recording') return;
    discarded = false;
    silentSince = 0;
    isSilent.value = false;
    const audio: MediaTrackConstraints | boolean = deviceId
      ? { deviceId: { exact: deviceId } }
      : true;
    stream = await navigator.mediaDevices.getUserMedia({ audio });
    audioCtx = new AudioContext();
    const source = audioCtx.createMediaStreamSource(stream);
    analyser = audioCtx.createAnalyser();
    analyser.fftSize = 512;
    source.connect(analyser);
    chunks = [];
    recorder = new MediaRecorder(stream);
    recorder.ondataavailable = (e) => {
      if (e.data.size > 0) chunks.push(e.data);
    };
    recorder.onstop = () => {
      void finalize();
    };
    recorder.start();
    startedAt = performance.now();
    state.value = 'recording';
    tick();
  }

  /** Stop and produce the recording. Rejects if decoding fails. */
  function stop(): Promise<Recording> {
    return new Promise<Recording>((resolve, reject) => {
      if (state.value !== 'recording' || !recorder) {
        reject(new Error('not recording'));
        return;
      }
      discarded = false;
      resolveStop = resolve;
      rejectStop = reject;
      recorder.stop();
    });
  }

  /** Abort recording, discarding the audio (no transcription). */
  function cancel() {
    if (state.value !== 'recording' || !recorder) return;
    discarded = true;
    recorder.stop();
  }

  onBeforeUnmount(cleanup);

  return { state, level, waveform, elapsedMs, isSilent, start, stop, cancel };
}
