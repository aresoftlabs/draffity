/**
 * Encode mono PCM (Float32, [-1,1]) to a 16-bit WAV byte buffer. Pure +
 * dependency-free, so it's unit-testable and reusable by both dictation and
 * voice notes. The backend ASR consumes 16 kHz mono WAV.
 */
export function encodeWav(samples: Float32Array, sampleRate: number): Uint8Array {
  const numChannels = 1;
  const bytesPerSample = 2;
  const blockAlign = numChannels * bytesPerSample;
  const byteRate = sampleRate * blockAlign;
  const dataSize = samples.length * blockAlign;

  const buffer = new ArrayBuffer(44 + dataSize);
  const view = new DataView(buffer);
  let p = 0;
  const writeStr = (s: string) => {
    for (let i = 0; i < s.length; i++) view.setUint8(p++, s.charCodeAt(i));
  };
  const u32 = (v: number) => {
    view.setUint32(p, v, true);
    p += 4;
  };
  const u16 = (v: number) => {
    view.setUint16(p, v, true);
    p += 2;
  };

  // RIFF header
  writeStr('RIFF');
  u32(36 + dataSize);
  writeStr('WAVE');
  // fmt chunk
  writeStr('fmt ');
  u32(16); // PCM chunk size
  u16(1); // PCM format
  u16(numChannels);
  u32(sampleRate);
  u32(byteRate);
  u16(blockAlign);
  u16(16); // bits per sample
  // data chunk
  writeStr('data');
  u32(dataSize);
  for (let i = 0; i < samples.length; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(p, s < 0 ? s * 0x8000 : s * 0x7fff, true);
    p += 2;
  }

  return new Uint8Array(buffer);
}
