import { Exports } from './types';

// An object used to compare prototype reference equality.
const __ref = {};

export function createExports(): Exports {
  return Object.create(__ref) as Exports;
}

export function isExports<T>(object: T): boolean {
  return Object.getPrototypeOf(object) === __ref;
}
