import { describe, expect, it } from 'vitest';
import { useVoiceRecorder } from './useVoiceRecorder';

describe('useVoiceRecorder', () => {
  it('starts idle with empty meters', () => {
    const r = useVoiceRecorder();
    expect(r.state.value).toBe('idle');
    expect(r.level.value).toBe(0);
    expect(r.elapsedMs.value).toBe(0);
    expect(r.isSilent.value).toBe(false);
    expect(r.waveform.value.length).toBe(0);
  });

  it('rejects stop() when not recording', async () => {
    const r = useVoiceRecorder();
    await expect(r.stop()).rejects.toThrow('not recording');
  });
});
