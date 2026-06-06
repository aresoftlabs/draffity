/** Float32 [-1,1] → Int16 PCM con clamp. */
export function float32ToInt16(input: Float32Array): Int16Array {
  const out = new Int16Array(input.length);
  for (let i = 0; i < input.length; i++) {
    let s = input[i];
    if (s > 1) s = 1;
    else if (s < -1) s = -1;
    out[i] = s < 0 ? s * 0x8000 : s * 0x7fff;
  }
  return out;
}

/** Downsample lineal de `inputRate` a 16 kHz (suficiente para voz). */
export function downsampleTo16k(input: Float32Array, inputRate: number): Float32Array {
  const target = 16000;
  if (inputRate === target) return input;
  const ratio = inputRate / target;
  const outLen = Math.floor(input.length / ratio);
  const out = new Float32Array(outLen);
  for (let i = 0; i < outLen; i++) {
    const srcIdx = i * ratio;
    const lo = Math.floor(srcIdx);
    const hi = Math.min(lo + 1, input.length - 1);
    const frac = srcIdx - lo;
    out[i] = input[lo] * (1 - frac) + input[hi] * frac;
  }
  return out;
}

/** RMS 0..1 de un Float32 (nivel para el medidor del overlay). */
export function rmsFloat32(input: Float32Array): number {
  if (input.length === 0) return 0;
  let sum = 0;
  for (const s of input) sum += s * s;
  return Math.sqrt(sum / input.length);
}
