import { describe, expect, it } from 'vitest';
import { shouldConfirmDiscard, LONG_RECORDING_MS } from './dictationDiscard';

describe('shouldConfirmDiscard', () => {
  it('does not confirm for short recordings', () => {
    expect(shouldConfirmDiscard(0)).toBe(false);
    expect(shouldConfirmDiscard(LONG_RECORDING_MS - 1)).toBe(false);
  });
  it('confirms once past the long-recording threshold', () => {
    expect(shouldConfirmDiscard(LONG_RECORDING_MS)).toBe(true);
    expect(shouldConfirmDiscard(60_000)).toBe(true);
  });
});
