import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import type { DictationContext } from './types';

vi.mock('@/services/ipc', () => ({
  ipc: {
    dictationStreamStart: vi.fn(async () => {}),
    dictationStreamFeed: vi.fn(async () => {}),
    dictationStreamStop: vi.fn(async () => {}),
    dictationStreamCancel: vi.fn(async () => {}),
  },
}));
const capture = {
  start: vi.fn(async () => {}),
  stop: vi.fn(async () => {}),
  level: { value: 0 },
  waveform: { value: new Uint8Array() },
  elapsedMs: { value: 0 },
  isSilent: { value: false },
};
vi.mock('@/audio/useStreamingCapture', () => ({ useStreamingCapture: () => capture }));
const handlers: Record<string, (e: { payload: unknown }) => void> = {};
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (name: string, cb: (e: { payload: unknown }) => void) => {
    handlers[name] = cb;
    return () => {};
  }),
}));

import { ipc } from '@/services/ipc';
import { createStreamingDictationMode } from './StreamingDictationMode';

function makeCtx(over: Partial<DictationContext> = {}): DictationContext {
  return {
    editor: {
      beginPending: vi.fn(),
      commit: vi.fn(() => true),
      clearPending: vi.fn(),
      setGhost: vi.fn(),
      clearGhost: vi.fn(),
      commitStreaming: vi.fn(() => true),
    },
    recorder: {} as unknown as DictationContext['recorder'],
    options: {},
    setPhase: vi.fn(),
    setProgress: vi.fn(),
    fail: vi.fn(),
    clipboardFallback: vi.fn(async () => {}),
    ...over,
  };
}

beforeEach(() => {
  setActivePinia(createPinia());
  for (const k in handlers) delete handlers[k];
});
afterEach(() => vi.clearAllMocks());

describe('StreamingDictationMode', () => {
  it('start opens the session, starts capture, goes recording', async () => {
    const mode = createStreamingDictationMode();
    const ctx = makeCtx();
    await mode.start(ctx);
    expect(ipc.dictationStreamStart).toHaveBeenCalledWith(16000);
    expect(capture.start).toHaveBeenCalled();
    expect(ctx.setPhase).toHaveBeenCalledWith('recording');
  });

  it('partial event sets ghost; final event commits and clears ghost', async () => {
    const mode = createStreamingDictationMode();
    const ctx = makeCtx();
    await mode.start(ctx);
    handlers['voice.stream.partial']?.({ payload: { text: 'hola mun' } });
    expect(ctx.editor.setGhost).toHaveBeenCalledWith('hola mun');
    handlers['voice.stream.final']?.({ payload: { text: 'hola mundo', seq: 0 } });
    expect(ctx.editor.clearGhost).toHaveBeenCalled();
    expect(ctx.editor.commitStreaming).toHaveBeenCalledWith('hola mundo ');
  });

  it('stop stops capture, closes session, clears ghost, goes idle', async () => {
    const mode = createStreamingDictationMode();
    const ctx = makeCtx();
    await mode.start(ctx);
    await mode.stop(ctx);
    expect(capture.stop).toHaveBeenCalled();
    expect(ipc.dictationStreamStop).toHaveBeenCalled();
    expect(ctx.editor.clearGhost).toHaveBeenCalled();
    expect(ctx.setPhase).toHaveBeenLastCalledWith('idle');
  });

  it('cancel stops capture, cancels session, goes idle', async () => {
    const mode = createStreamingDictationMode();
    const ctx = makeCtx();
    await mode.start(ctx);
    mode.cancel(ctx);
    expect(capture.stop).toHaveBeenCalled();
    expect(ipc.dictationStreamCancel).toHaveBeenCalled();
    expect(ctx.setPhase).toHaveBeenLastCalledWith('idle');
  });
});
