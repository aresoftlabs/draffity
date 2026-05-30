import { describe, expect, it } from 'vitest';
import { replaceById } from './helpers';

describe('replaceById', () => {
  it('replaces the element with the matching id in place', () => {
    const list = [
      { id: 'a', n: 1 },
      { id: 'b', n: 2 },
    ];
    replaceById(list, 'b', { id: 'b', n: 99 });
    expect(list).toEqual([
      { id: 'a', n: 1 },
      { id: 'b', n: 99 },
    ]);
  });

  it('is a no-op when the id is absent', () => {
    const list = [{ id: 'a', n: 1 }];
    replaceById(list, 'x', { id: 'x', n: 5 });
    expect(list).toEqual([{ id: 'a', n: 1 }]);
  });
});
