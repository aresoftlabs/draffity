import { describe, expect, it } from 'vitest';
import { isWireError } from './useIpcError';

describe('isWireError', () => {
  it('returns true for a valid WireError', () => {
    expect(isWireError({ code: 'E001', message: 'fail' })).toBe(true);
  });

  it('returns false for null', () => {
    expect(isWireError(null)).toBe(false);
  });

  it('returns false for a plain string', () => {
    expect(isWireError('error')).toBe(false);
  });

  it('returns false for a plain object without code or message', () => {
    expect(isWireError({ foo: 'bar' })).toBe(false);
  });

  it('returns false for undefined', () => {
    expect(isWireError(undefined)).toBe(false);
  });

  it('returns false for a number', () => {
    expect(isWireError(42)).toBe(false);
  });
});
