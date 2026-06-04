import { describe, expect, it, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import VoiceWaveform from './VoiceWaveform.vue';

describe('VoiceWaveform', () => {
  it('renders a canvas', () => {
    const w = mount(VoiceWaveform, { props: { data: new Uint8Array([128, 130, 126]) } });
    expect(w.find('canvas').exists()).toBe(true);
  });

  it('draws when data changes (guards when ctx is unavailable)', async () => {
    const ctx = {
      clearRect: vi.fn(),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      lineWidth: 0,
      strokeStyle: '',
    };
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(
      ctx as unknown as CanvasRenderingContext2D,
    );
    const w = mount(VoiceWaveform, { props: { data: new Uint8Array([128, 200, 60, 140]) } });
    await w.setProps({ data: new Uint8Array([128, 64, 200, 100]) });
    expect(ctx.stroke).toHaveBeenCalled();
  });
});
