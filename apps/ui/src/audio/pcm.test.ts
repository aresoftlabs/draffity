import { describe, expect, it } from 'vitest';
import { float32ToInt16, downsampleTo16k, rmsFloat32 } from './pcm';

describe('pcm helpers', () => {
  it('float32ToInt16 clamps and scales', () => {
    const out = float32ToInt16(new Float32Array([0, 1, -1, 2, -2]));
    expect(out[0]).toBe(0);
    expect(out[1]).toBe(32767);
    expect(out[2]).toBe(-32768);
    expect(out[3]).toBe(32767); // clamp >1
    expect(out[4]).toBe(-32768); // clamp <-1
  });
  it('downsampleTo16k halves length from 32k', () => {
    const input = new Float32Array(320);
    const out = downsampleTo16k(input, 32000);
    expect(out.length).toBe(160);
  });
  it('downsampleTo16k is identity at 16k', () => {
    const input = new Float32Array([0.1, 0.2]);
    expect(downsampleTo16k(input, 16000)).toBe(input);
  });
  it('rmsFloat32 is 0 for silence, ~1 for full-scale square', () => {
    expect(rmsFloat32(new Float32Array(10))).toBe(0);
    expect(rmsFloat32(new Float32Array([1, -1, 1, -1]))).toBeCloseTo(1);
  });
});
