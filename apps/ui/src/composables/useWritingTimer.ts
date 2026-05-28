import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { useBeepAudio } from './useBeepAudio';
import { useTimerStorage } from './useTimerStorage';

/**
 * Pomodoro-style writing timer with two configurable phases (`work` and
 * `break`). Tick granularity is 1s. When a phase completes the timer
 * auto-transitions to the next phase and plays a short beep.
 *
 * State machine
 *   idle → start() → work → (timer ends) → break → (timer ends) → work …
 *   any → pause()  → paused (same phase, frozen remainingMs)
 *   any → resume() → previous phase resumes from remainingMs
 *   any → reset()  → idle (phase=work, full focusMin)
 */
export type TimerPhase = 'idle' | 'work' | 'break' | 'paused';

export function useWritingTimer() {
  const storage = useTimerStorage();
  const beep = useBeepAudio();

  const focusMin = ref(storage.load('focusMin', 25));
  const breakMin = ref(storage.load('breakMin', 5));

  const phase = ref<TimerPhase>('idle');
  /** Phase that was running before pause; used to know what to resume. */
  const pausedFrom = ref<'work' | 'break'>('work');
  const remainingMs = ref(focusMin.value * 60_000);
  const sessionsCompleted = ref(0);

  let interval: ReturnType<typeof setInterval> | null = null;
  let phaseEndsAt = 0;

  watch(focusMin, (v) => {
    storage.save('focusMin', v);
    if (phase.value === 'idle') remainingMs.value = v * 60_000;
  });
  watch(breakMin, (v) => storage.save('breakMin', v));

  function clearInterval_() {
    if (interval !== null) {
      clearInterval(interval);
      interval = null;
    }
  }

  function tick() {
    const now = Date.now();
    const left = phaseEndsAt - now;
    if (left <= 0) {
      clearInterval_();
      remainingMs.value = 0;
      onPhaseComplete();
    } else {
      remainingMs.value = left;
    }
  }

  function onPhaseComplete() {
    beep.play();
    if (phase.value === 'work') {
      sessionsCompleted.value += 1;
      enterPhase('break');
    } else if (phase.value === 'break') {
      enterPhase('work');
    }
  }

  function enterPhase(next: 'work' | 'break') {
    const mins = next === 'work' ? focusMin.value : breakMin.value;
    const ms = mins * 60_000;
    phase.value = next;
    remainingMs.value = ms;
    phaseEndsAt = Date.now() + ms;
    clearInterval_();
    interval = setInterval(tick, 1000);
  }

  function start() {
    if (phase.value === 'work' || phase.value === 'break') return;
    if (phase.value === 'paused') {
      // Resume from the same remaining time on the previous phase.
      phase.value = pausedFrom.value;
      phaseEndsAt = Date.now() + remainingMs.value;
      clearInterval_();
      interval = setInterval(tick, 1000);
      return;
    }
    enterPhase('work');
  }

  function pause() {
    if (phase.value !== 'work' && phase.value !== 'break') return;
    pausedFrom.value = phase.value;
    phase.value = 'paused';
    clearInterval_();
  }

  function reset() {
    clearInterval_();
    phase.value = 'idle';
    sessionsCompleted.value = 0;
    remainingMs.value = focusMin.value * 60_000;
  }

  function skip() {
    if (phase.value === 'work') {
      sessionsCompleted.value += 1;
      enterPhase('break');
    } else if (phase.value === 'break' || phase.value === 'paused') {
      enterPhase('work');
    }
  }

  const running = computed(() => phase.value === 'work' || phase.value === 'break');
  const display = computed(() => {
    const totalSec = Math.max(0, Math.ceil(remainingMs.value / 1000));
    const m = Math.floor(totalSec / 60);
    const s = totalSec % 60;
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  });

  onBeforeUnmount(() => clearInterval_());

  return {
    focusMin,
    breakMin,
    phase,
    remainingMs,
    sessionsCompleted,
    running,
    display,
    start,
    pause,
    reset,
    skip,
  };
}
