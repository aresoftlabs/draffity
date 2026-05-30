import { afterEach, describe, expect, it, vi } from 'vitest';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useDictation } from './useDictation';

// Stub the audio recorder so we can drive the success/failure of start()/stop()
// without real MediaRecorder APIs (absent in the test DOM).
const { recorder } = vi.hoisted(() => ({
  recorder: {
    start: vi.fn(),
    stop: vi.fn(),
    cancel: vi.fn(),
    level: { value: 0 },
  },
}));
vi.mock('@/audio/useAudioRecorder', () => ({ useAudioRecorder: () => recorder }));

const invokeMock = vi.mocked(invoke);

describe('useDictation', () => {
  afterEach(() => {
    invokeMock.mockReset();
    recorder.start.mockReset();
    recorder.stop.mockReset();
    recorder.cancel.mockReset();
  });

  it('reports a start failure through onError, not just the silent ref', async () => {
    recorder.start.mockRejectedValueOnce(new Error('microphone denied'));
    const onError = vi.fn();
    const d = useDictation(ref(null), { onError });

    await d.start();

    expect(d.phase.value).toBe('idle');
    expect(d.error.value).toBe('microphone denied');
    expect(onError).toHaveBeenCalledWith('microphone denied');
  });

  it('reports a transcription failure through onError', async () => {
    recorder.start.mockResolvedValueOnce(undefined);
    recorder.stop.mockResolvedValueOnce({ wav: new Uint8Array() });
    invokeMock.mockRejectedValueOnce(new Error('asr engine missing'));
    const onError = vi.fn();
    const d = useDictation(ref(null), { onError });

    await d.start();
    await d.stopAndInsert();

    expect(d.error.value).toBe('asr engine missing');
    expect(onError).toHaveBeenCalledWith('asr engine missing');
  });
});
