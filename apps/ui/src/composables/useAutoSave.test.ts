import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { defineComponent, h, nextTick } from 'vue';
import { mount } from '@vue/test-utils';
import { useAutoSave } from './useAutoSave';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

function host(saveFn: () => void | Promise<void>, delay: number | (() => number) = 100) {
  type Saver = ReturnType<typeof useAutoSave>;
  const captured: { saver: Saver | null } = { saver: null };
  const Comp = defineComponent({
    setup() {
      captured.saver = useAutoSave(saveFn, delay);
      return () => h('div');
    },
  });
  const wrapper = mount(Comp);
  return {
    wrapper,
    get saver(): Saver {
      if (!captured.saver) throw new Error('saver not initialized');
      return captured.saver;
    },
  };
}

describe('useAutoSave', () => {
  it('calls save once after debounce window', async () => {
    const save = vi.fn();
    const { saver } = host(save, 200);

    saver.trigger();
    saver.trigger();
    saver.trigger();
    expect(save).not.toHaveBeenCalled();

    vi.advanceTimersByTime(199);
    expect(save).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    await nextTick();
    expect(save).toHaveBeenCalledTimes(1);
  });

  it('flush runs immediately and clears pending', async () => {
    const save = vi.fn();
    const { saver } = host(save, 500);

    saver.trigger();
    expect(saver.pending()).toBe(true);

    await saver.flush();
    expect(save).toHaveBeenCalledTimes(1);
    expect(saver.pending()).toBe(false);
  });

  it('cancel discards a pending invocation', async () => {
    const save = vi.fn();
    const { saver } = host(save, 200);

    saver.trigger();
    saver.cancel();
    vi.advanceTimersByTime(500);
    await nextTick();
    expect(save).not.toHaveBeenCalled();
  });

  it('reads a dynamic delay from a getter on each trigger', async () => {
    const save = vi.fn();
    let ms = 200;
    const { saver } = host(save, () => ms);

    saver.trigger();
    vi.advanceTimersByTime(200);
    await nextTick();
    expect(save).toHaveBeenCalledTimes(1);

    // Raising the configured delay takes effect on the next debounce window.
    ms = 500;
    saver.trigger();
    vi.advanceTimersByTime(200);
    await nextTick();
    expect(save).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(300);
    await nextTick();
    expect(save).toHaveBeenCalledTimes(2);
  });

  it('flushes pending save on unmount', async () => {
    const save = vi.fn();
    const { wrapper, saver } = host(save, 500);

    saver.trigger();
    expect(save).not.toHaveBeenCalled();

    wrapper.unmount();
    await nextTick();
    // onBeforeUnmount triggers a flush — invoke must run.
    expect(save).toHaveBeenCalledTimes(1);
  });
});
