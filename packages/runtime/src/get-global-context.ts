/* eslint-disable no-nested-ternary -- allow */
import type { Global } from './types';

export function getGlobalContext(): Global {
  return typeof globalThis !== 'undefined'
    ? globalThis
    : typeof global !== 'undefined'
      ? global
      : typeof window !== 'undefined'
        ? window
        : this;
}
