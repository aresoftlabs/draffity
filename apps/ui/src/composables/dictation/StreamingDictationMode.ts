import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { ipc } from '@/services/ipc';
import type { VoiceStreamPartial, VoiceStreamFinal } from '@/services/ipc';
import { useStreamingCapture } from '@/audio/useStreamingCapture';
import { resolveInputDeviceId } from './settings';
import type { DictationMode } from './types';

const SAMPLE_RATE = 16000;

/**
 * Modo "En vivo": abre la sesión de streaming en backend, captura PCM por
 * AudioWorklet y lo va enviando; los partials pintan el fantasma gris y cada
 * final consolida una frase. El documento solo recibe texto confirmado.
 */
export function createStreamingDictationMode() {
  const capture = useStreamingCapture();
  let unlistenPartial: UnlistenFn | null = null;
  let unlistenFinal: UnlistenFn | null = null;
  let active = false;

  async function teardown() {
    active = false;
    unlistenPartial?.();
    unlistenFinal?.();
    unlistenPartial = null;
    unlistenFinal = null;
  }

  const mode: DictationMode = {
    id: 'streaming',

    async start(ctx) {
      try {
        await ipc.dictationStreamStart(SAMPLE_RATE);
        active = true;
        unlistenPartial = await listen<VoiceStreamPartial>('voice:stream:partial', (e) => {
          if (active) ctx.editor.setGhost(e.payload.text);
        });
        unlistenFinal = await listen<VoiceStreamFinal>('voice:stream:final', (e) => {
          if (!active) return;
          ctx.editor.clearGhost();
          const text = e.payload.text.trim();
          if (text) ctx.editor.commitStreaming(`${text} `);
        });
        await capture.start(resolveInputDeviceId(), (pcm) => {
          void ipc.dictationStreamFeed(pcm);
        });
        ctx.setPhase('recording');
      } catch (e) {
        await capture.stop();
        await teardown();
        ctx.fail(e);
        ctx.setPhase('idle');
      }
    },

    async stop(ctx) {
      if (!active) return;
      ctx.setPhase('transcribing');
      try {
        await capture.stop();
        const finals = await ipc.dictationStreamStop();
        ctx.editor.clearGhost();
        for (const f of finals) {
          const text = f.text.trim();
          if (text) ctx.editor.commitStreaming(`${text} `);
        }
      } catch (e) {
        ctx.fail(e);
      } finally {
        await teardown();
        ctx.editor.clearGhost();
        ctx.setPhase('idle');
      }
    },

    cancel(ctx) {
      if (!active) return;
      void capture.stop();
      void ipc.dictationStreamCancel().catch(() => {});
      void teardown();
      ctx.editor.clearGhost();
      ctx.setPhase('idle');
    },
  };

  // El host accede a `capture` para reflejar level/waveform/elapsedMs en el overlay.
  return Object.assign(mode, { capture });
}
