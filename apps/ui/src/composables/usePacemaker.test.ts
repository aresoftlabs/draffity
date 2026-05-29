import { describe, expect, it } from 'vitest';
import { computePacemaker } from './usePacemaker';

const NOW = 1_700_000_000_000;
const DAY = 24 * 60 * 60 * 1000;

describe('computePacemaker', () => {
  it('is inactive without a goal or deadline', () => {
    expect(
      computePacemaker({
        goal: null,
        current: 0,
        deadline: NOW + DAY,
        wordsThisSession: 0,
        now: NOW,
      }).active,
    ).toBe(false);
    expect(
      computePacemaker({ goal: 1000, current: 0, deadline: null, wordsThisSession: 0, now: NOW })
        .active,
    ).toBe(false);
  });

  it('computes 1000 words/day for 30k remaining over 30 days', () => {
    const p = computePacemaker({
      goal: 30_000,
      current: 0,
      deadline: NOW + 30 * DAY,
      wordsThisSession: 0,
      now: NOW,
    });
    expect(p.active).toBe(true);
    expect(p.wordsPerDay).toBe(1000);
    expect(p.daysRemaining).toBe(30);
  });

  it('is ontrack when the session already met the daily pace', () => {
    const p = computePacemaker({
      goal: 30_000,
      current: 0,
      deadline: NOW + 30 * DAY,
      wordsThisSession: 1000,
      now: NOW,
    });
    expect(p.status).toBe('ontrack');
  });

  it('is close at half the pace and behind below half', () => {
    const base = { goal: 30_000, current: 0, deadline: NOW + 30 * DAY, now: NOW };
    expect(computePacemaker({ ...base, wordsThisSession: 600 }).status).toBe('close');
    expect(computePacemaker({ ...base, wordsThisSession: 100 }).status).toBe('behind');
  });

  it('reports done when the goal is reached', () => {
    const p = computePacemaker({
      goal: 1000,
      current: 1000,
      deadline: NOW + DAY,
      wordsThisSession: 0,
      now: NOW,
    });
    expect(p.status).toBe('done');
    expect(p.wordsPerDay).toBe(0);
  });

  it('is overdue when the deadline passed with words owed', () => {
    const p = computePacemaker({
      goal: 1000,
      current: 200,
      deadline: NOW - DAY,
      wordsThisSession: 0,
      now: NOW,
    });
    expect(p.status).toBe('overdue');
    expect(p.wordsRemaining).toBe(800);
  });
});
