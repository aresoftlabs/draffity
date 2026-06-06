import { ipc } from '@/services/ipc';
import { shouldConfirmDiscard } from '@/composables/dictationDiscard';
import { resolveInputDeviceId, resolveVoiceLanguage } from './settings';
import type { DictationMode } from './types';

/**
 * Modo manual (comportamiento actual): graba, al parar transcribe el clip
 * completo vía `transcribe_audio` e inserta el resultado en el cursor. El
 * `runId` interno invalida una transcripción en vuelo si se cancela.
 */
export function createManualDictationMode(): DictationMode {
  let runId = 0;

  return {
    id: 'manual',

    async start(ctx) {
      try {
        await ctx.recorder.start(resolveInputDeviceId());
        ctx.setPhase('recording');
      } catch (e) {
        ctx.fail(e);
        ctx.setPhase('idle');
      }
    },

    async stop(ctx) {
      const myRun = ++runId;
      ctx.editor.beginPending();
      ctx.setProgress(0);
      ctx.setPhase('transcribing');
      try {
        const rec = await ctx.recorder.stop();
        const transcript = await ipc.transcribeAudio(rec.wav, undefined, resolveVoiceLanguage());
        if (myRun !== runId) return; // cancelado mientras transcribía
        const clean = transcript.text.trim();
        if (!clean) {
          ctx.editor.clearPending();
        } else {
          const ok = ctx.editor.commit(`${clean} `);
          if (!ok) await ctx.clipboardFallback(clean);
        }
      } catch (e) {
        if (myRun === runId) {
          ctx.editor.clearPending();
          ctx.fail(e);
        }
      } finally {
        if (myRun === runId) {
          ctx.setPhase('idle');
          ctx.setProgress(null);
        }
      }
    },

    cancel(ctx) {
      if (
        shouldConfirmDiscard(ctx.recorder.elapsedMs.value) &&
        !(ctx.options.confirmDiscard?.() ?? true)
      ) {
        return;
      }
      runId++; // invalida cualquier transcripción en vuelo
      ctx.recorder.cancel();
      ctx.editor.clearPending();
      ctx.setPhase('idle');
      ctx.setProgress(null);
    },
  };
}
