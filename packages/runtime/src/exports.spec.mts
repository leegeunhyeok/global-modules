import { describe, it, expect } from 'vitest';
import { createExports, isExports } from './exports.js';

describe('exports', () => {
  it('should match snapshot', () => {
    expect(createExports()).toMatchInlineSnapshot(`{}`);
  });

  describe('isExports', () => {
    it('should return `true` if provided object was created using `createExports()`', () => {
      const value = createExports();
      expect(isExports(value)).toBe(true);
    });

    it('should return `false` if provided value is plain object', () => {
      const value = {};
      expect(isExports(value)).toBe(false);
    });
  });
});
