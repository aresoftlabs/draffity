import { describe, expect, it } from 'vitest';
import { encodeWav } from './wav';

function ascii(bytes: Uint8Array, offset: number, len: number): string {
  return String.fromCharCode(...bytes.slice(offset, offset + len));
}

describe('encodeWav', () => {
  it('writes a 44-byte header plus 2 bytes per sample', () => {
    const wav = encodeWav(new Float32Array(4), 16000);
    expect(wav.length).toBe(44 + 4 * 2);
  });

  it('has RIFF/WAVE/fmt/data chunk markers', () => {
    const wav = encodeWav(new Float32Array(1), 16000);
    expect(ascii(wav, 0, 4)).toBe('RIFF');
    expect(ascii(wav, 8, 4)).toBe('WAVE');
    expect(ascii(wav, 12, 4)).toBe('fmt ');
    expect(ascii(wav, 36, 4)).toBe('data');
  });

  it('encodes sample rate, mono channel and 16-bit depth', () => {
    const wav = encodeWav(new Float32Array(2), 16000);
    const view = new DataView(wav.buffer);
    expect(view.getUint16(22, true)).toBe(1); // channels
    expect(view.getUint32(24, true)).toBe(16000); // sample rate
    expect(view.getUint16(34, true)).toBe(16); // bits per sample
  });

  it('clamps and quantises samples to int16', () => {
    const wav = encodeWav(new Float32Array([1, -1, 2, -2]), 16000);
    const view = new DataView(wav.buffer);
    expect(view.getInt16(44, true)).toBe(0x7fff); // +1.0 → max
    expect(view.getInt16(46, true)).toBe(-0x8000); // -1.0 → min
    expect(view.getInt16(48, true)).toBe(0x7fff); // clamped from +2
    expect(view.getInt16(50, true)).toBe(-0x8000); // clamped from -2
  });
});
