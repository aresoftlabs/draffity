/** Pacemaker (J-03): how many words/day are needed to hit the project goal by
 *  the deadline, plus a status that reflects whether the current session is
 *  keeping up. Pure + framework-free so it unit-tests without a DOM. */

export type PacemakerStatus =
  | 'none' // no goal and/or no deadline → pacemaker inactive
  | 'done' // goal already reached
  | 'ontrack' // this session already hit today's needed pace (green)
  | 'close' // at least halfway to today's pace (amber)
  | 'behind' // below half of today's pace (red)
  | 'overdue'; // deadline already passed with words still owed (red)

export interface Pacemaker {
  active: boolean;
  status: PacemakerStatus;
  /** Words/day needed from now to hit the goal by the deadline. */
  wordsPerDay: number;
  /** Whole calendar days from `now` to the deadline (can be 0; <0 = overdue). */
  daysRemaining: number;
  wordsRemaining: number;
}

const DAY_MS = 24 * 60 * 60 * 1000;

export interface PacemakerInput {
  goal: number | null | undefined;
  current: number;
  deadline: number | null | undefined;
  /** Words written in the current session (proxy for "today's effort"). */
  wordsThisSession: number;
  /** Injectable clock for testing; defaults to `Date.now()`. */
  now?: number;
}

const INACTIVE: Pacemaker = {
  active: false,
  status: 'none',
  wordsPerDay: 0,
  daysRemaining: 0,
  wordsRemaining: 0,
};

export function computePacemaker(input: PacemakerInput): Pacemaker {
  const { goal, current, deadline, wordsThisSession } = input;
  const now = input.now ?? Date.now();

  if (!goal || goal <= 0 || deadline == null) return INACTIVE;

  const wordsRemaining = Math.max(0, goal - current);
  const daysRemaining = Math.ceil((deadline - now) / DAY_MS);

  if (wordsRemaining === 0) {
    return { active: true, status: 'done', wordsPerDay: 0, daysRemaining, wordsRemaining: 0 };
  }
  if (daysRemaining <= 0) {
    // Past the deadline with words still owed.
    return {
      active: true,
      status: 'overdue',
      wordsPerDay: wordsRemaining,
      daysRemaining,
      wordsRemaining,
    };
  }

  const wordsPerDay = Math.ceil(wordsRemaining / daysRemaining);
  let status: PacemakerStatus;
  if (wordsThisSession >= wordsPerDay) status = 'ontrack';
  else if (wordsThisSession >= wordsPerDay / 2) status = 'close';
  else status = 'behind';

  return { active: true, status, wordsPerDay, daysRemaining, wordsRemaining };
}
