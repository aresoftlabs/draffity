import { afterEach, describe, expect, it, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import type { DictationContext } from './types';
import { createManualDictationMode } from './ManualDictationMode';

vi.mock('@/services/ipc', () => ({
  ipc: { transcribeAudio: vi.fn() },
}));
import { ipc } from '@/services/ipc';

function makeCtx(over: Partial<DictationContext> = {}): DictationContext {
  return {
    editor: { beginPending: vi.fn(), commit: vi.fn(() => true), clearPending: vi.fn() },
    recorder: {
      start: vi.fn(async () => {}),
      stop: vi.fn(async () => ({ wav: new Uint8Array(), durationMs: 1000 })),
      cancel: vi.fn(),
      elapsedMs: { value: 0 },
    } as unknown as DictationContext['recorder'],
    options: {},
    setPhase: vi.fn(),
    setProgress: vi.fn(),
    fail: vi.fn(),
    clipboardFallback: vi.fn(async () => {}),
    ...over,
  };
}

beforeEach(() => setActivePinia(createPinia()));
afterEach(() => vi.clearAllMocks());

describe('ManualDictationMode', () => {
  it('start: records and moves to recording', async () => {
    const mode = createManualDictationMode();
    const ctx = makeCtx();
    await mode.start(ctx);
    expect(ctx.recorder.start).toHaveBeenCalled();
    expect(ctx.setPhase).toHaveBeenCalledWith('recording');
  });

  it('stop: transcribes and commits the text plus a trailing space', async () => {
    (ipc.transcribeAudio as ReturnType<typeof vi.fn>).mockResolvedValue({
      text: 'hola mundo',
      segments: [],
    });
    const mode = createManualDictationMode();
    const ctx = makeCtx();
    await mode.start(ctx);
    await mode.stop(ctx);
    expect(ctx.editor.beginPending).toHaveBeenCalled();
    expect(ctx.editor.commit).toHaveBeenCalledWith('hola mundo ');
    expect(ctx.setPhase).toHaveBeenLastCalledWith('idle');
  });

  it('stop: empty transcript clears the pending marker, no commit', async () => {
    (ipc.transcribeAudio as ReturnType<typeof vi.fn>).mockResolvedValue({
      text: '   ',
      segments: [],
    });
    const mode = createManualDictationMode();
    const ctx = makeCtx();
    await mode.start(ctx);
    await mode.stop(ctx);
    expect(ctx.editor.clearPending).toHaveBeenCalled();
    expect(ctx.editor.commit).not.toHaveBeenCalled();
  });

  it('stop: when commit fails, falls back to clipboard', async () => {
    (ipc.transcribeAudio as ReturnType<typeof vi.fn>).mockResolvedValue({
      text: 'hola',
      segments: [],
    });
    const mode = createManualDictationMode();
    const ctx = makeCtx({
      editor: { beginPending: vi.fn(), commit: vi.fn(() => false), clearPending: vi.fn() },
    });
    await mode.start(ctx);
    await mode.stop(ctx);
    expect(ctx.clipboardFallback).toHaveBeenCalledWith('hola');
  });

  it('cancel after stop invalidates the in-flight transcription (no commit)', async () => {
    let resolveT: (v: { text: string; segments: [] }) => void = () => {};
    (ipc.transcribeAudio as ReturnType<typeof vi.fn>).mockReturnValue(
      new Promise((r) => {
        resolveT = r;
      }),
    );
    const mode = createManualDictationMode();
    const ctx = makeCtx();
    await mode.start(ctx);
    const stopping = mode.stop(ctx);
    mode.cancel(ctx); // invalida el run en vuelo
    resolveT({ text: 'tarde', segments: [] });
    await stopping;
    expect(ctx.editor.commit).not.toHaveBeenCalledWith('tarde ');
    expect(ctx.recorder.cancel).toHaveBeenCalled();
  });
});
