-- Session/daily targets (J-04): record, per day, the writer's word goal and
-- whether it was met. `goal_words` snapshots the goal in force that day (NULL
-- when no goal was set); `goal_met` is 1 once the day's accumulated words reach
-- it. Powers the "goal-met streak" (J-05). Additive, non-destructive.

ALTER TABLE daily_writing ADD COLUMN goal_words INTEGER;
ALTER TABLE daily_writing ADD COLUMN goal_met INTEGER NOT NULL DEFAULT 0;
