import { ref } from 'vue';
import { downsampleTo16k, float32ToInt16, rmsFloat32 } from './pcm';

/**
 * Captura de micrófono en streaming vía AudioWorklet. Emite lotes PCM16 16 kHz
 * mono al callback `onPcm` y expone `level/waveform/elapsedMs/isSilent` para el
 * overlay (misma superficie reactiva que el recorder manual). No produce un WAV;
 * es para el dictado en vivo. jsdom no tiene AudioWorklet → se valida en build.
 */
export function useStreamingCapture() {
  const level = ref(0);
  const waveform = ref<Uint8Array>(new Uint8Array(0));
  const elapsedMs = ref(0);
  const isSilent = ref(false);

  let stream: MediaStream | null = null;
  let ctx: AudioContext | null = null;
  let source: MediaStreamAudioSourceNode | null = null;
  let node: AudioWorkletNode | null = null;
  let startedAt = 0;
  let onPcm: ((pcm: Int16Array) => void) | null = null;

  async function start(deviceId: string | null, cb: (pcm: Int16Array) => void) {
    onPcm = cb;
    const audio: MediaTrackConstraints | boolean = deviceId
      ? { deviceId: { exact: deviceId }, channelCount: 1 }
      : { channelCount: 1 };
    stream = await navigator.mediaDevices.getUserMedia({ audio });
    let needsResample = false;
    try {
      ctx = new AudioContext({ sampleRate: 16000, latencyHint: 'interactive' });
      needsResample = Math.abs(ctx.sampleRate - 16000) > 1;
    } catch {
      ctx = new AudioContext({ latencyHint: 'interactive' });
      needsResample = true;
    }
    if (ctx.state === 'suspended') await ctx.resume();
    await ctx.audioWorklet.addModule('/pcm-worklet.js');
    source = ctx.createMediaStreamSource(stream);
    node = new AudioWorkletNode(ctx, 'pcm-capture', {
      numberOfOutputs: 0,
      channelCount: 1,
    } as AudioWorkletNodeOptions);
    const nativeRate = ctx.sampleRate;
    node.port.onmessage = (e: MessageEvent<{ pcm: ArrayBuffer }>) => {
      let f32: Float32Array = new Float32Array(e.data.pcm as ArrayBuffer);
      if (needsResample) f32 = downsampleTo16k(f32, nativeRate);
      level.value = rmsFloat32(f32);
      elapsedMs.value = performance.now() - startedAt;
      onPcm?.(float32ToInt16(f32));
    };
    source.connect(node);
    startedAt = performance.now();
  }

  async function stop() {
    try {
      source?.disconnect();
    } catch {
      /* noop */
    }
    try {
      node?.disconnect();
    } catch {
      /* noop */
    }
    stream?.getTracks().forEach((t) => t.stop());
    try {
      await ctx?.close();
    } catch {
      /* noop */
    }
    source = null;
    node = null;
    stream = null;
    ctx = null;
    onPcm = null;
    level.value = 0;
    waveform.value = new Uint8Array(0);
    elapsedMs.value = 0;
    isSilent.value = false;
  }

  return { level, waveform, elapsedMs, isSilent, start, stop };
}
