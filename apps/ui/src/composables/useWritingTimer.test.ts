import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { nextTick } from 'vue';
import { useWritingTimer } from './useWritingTimer';

// Setup-only: ensure each test starts with a clean localStorage + fake timers.
beforeEach(() => {
  localStorage.clear();
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('useWritingTimer', () => {
  it('starts in idle with default focus duration', () => {
    const t = useWritingTimer();
    expect(t.phase.value).toBe('idle');
    expect(t.focusMin.value).toBe(25);
    expect(t.breakMin.value).toBe(5);
    expect(t.remainingMs.value).toBe(25 * 60_000);
    expect(t.running.value).toBe(false);
  });

  it('transitions idle → work on start()', () => {
    const t = useWritingTimer();
    t.start();
    expect(t.phase.value).toBe('work');
    expect(t.running.value).toBe(true);
    expect(t.remainingMs.value).toBe(25 * 60_000);
  });

  it('counts down while running', () => {
    const t = useWritingTimer();
    t.start();
    vi.advanceTimersByTime(5_000);
    expect(t.remainingMs.value).toBeLessThanOrEqual(25 * 60_000 - 4_000);
    expect(t.remainingMs.value).toBeGreaterThan(25 * 60_000 - 6_000);
  });

  it('auto-transitions to break when work phase ends', () => {
    const t = useWritingTimer();
    t.focusMin.value = 1;
    t.start();
    vi.advanceTimersByTime(61_000); // a touch over 1 min
    expect(t.phase.value).toBe('break');
    expect(t.sessionsCompleted.value).toBe(1);
  });

  it('pause() freezes remaining time; resume continues from there', () => {
    const t = useWritingTimer();
    t.focusMin.value = 2;
    t.start();
    vi.advanceTimersByTime(30_000);
    const remBeforePause = t.remainingMs.value;
    t.pause();
    expect(t.phase.value).toBe('paused');
    expect(t.running.value).toBe(false);

    // Wall-clock advances but remaining shouldn't change while paused.
    vi.advanceTimersByTime(10_000);
    expect(t.remainingMs.value).toBe(remBeforePause);

    t.start(); // resume
    expect(t.phase.value).toBe('work');
    vi.advanceTimersByTime(5_000);
    expect(t.remainingMs.value).toBeLessThan(remBeforePause);
  });

  it('reset() returns to idle and clears sessions', () => {
    const t = useWritingTimer();
    t.focusMin.value = 1;
    t.start();
    vi.advanceTimersByTime(61_000); // 1 work session completes
    expect(t.sessionsCompleted.value).toBe(1);
    t.reset();
    expect(t.phase.value).toBe('idle');
    expect(t.sessionsCompleted.value).toBe(0);
    expect(t.remainingMs.value).toBe(60_000); // focus = 1 min
  });

  it('skip() in work advances to break and bumps sessions', () => {
    const t = useWritingTimer();
    t.start();
    expect(t.phase.value).toBe('work');
    t.skip();
    expect(t.phase.value).toBe('break');
    expect(t.sessionsCompleted.value).toBe(1);
  });

  it('skip() in break advances to work without incrementing sessions', () => {
    const t = useWritingTimer();
    t.focusMin.value = 1;
    t.start();
    vi.advanceTimersByTime(61_000); // → break, sessions=1
    expect(t.phase.value).toBe('break');
    expect(t.sessionsCompleted.value).toBe(1);
    t.skip();
    expect(t.phase.value).toBe('work');
    expect(t.sessionsCompleted.value).toBe(1);
  });

  it('display formats remaining as MM:SS with zero padding', () => {
    const t = useWritingTimer();
    t.focusMin.value = 5;
    t.start();
    expect(t.display.value).toBe('05:00');
    vi.advanceTimersByTime(1_000);
    expect(t.display.value).toMatch(/^04:5[89]$/);
  });

  it('start() while running is a no-op', () => {
    const t = useWritingTimer();
    t.start();
    const phase = t.phase.value;
    const rem = t.remainingMs.value;
    t.start();
    expect(t.phase.value).toBe(phase);
    expect(t.remainingMs.value).toBe(rem);
  });

  it('changing focusMin while idle updates remaining after the watcher flushes', async () => {
    const t = useWritingTimer();
    t.focusMin.value = 10;
    await nextTick();
    expect(t.remainingMs.value).toBe(10 * 60_000);
  });
});
