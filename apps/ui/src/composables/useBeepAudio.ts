/**
 * WebAudio beep generator. Stateless composable returned as an object so
 * future enhancements (volume, waveform from settings) have a place to live
 * without changing the call sites.
 */
export function useBeepAudio() {
  function play(durationMs = 220, freq = 880) {
    try {
      const w = window as unknown as {
        AudioContext?: typeof AudioContext;
        webkitAudioContext?: typeof AudioContext;
      };
      const Ctor = w.AudioContext ?? w.webkitAudioContext;
      if (!Ctor) return;
      const ctx = new Ctor();
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.frequency.value = freq;
      osc.type = 'sine';
      osc.connect(gain);
      gain.connect(ctx.destination);
      gain.gain.value = 0.08;
      osc.start();
      setTimeout(() => {
        try {
          osc.stop();
          void ctx.close();
        } catch {
          // already stopped
        }
      }, durationMs);
    } catch {
      /* audio blocked — silent failure */
    }
  }

  return { play };
}
