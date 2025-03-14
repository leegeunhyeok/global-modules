import type { Exports } from './types';

export function interopDefaultExport<T extends { exports: Exports }>(
  module: T,
) {
  if (typeof module.exports.default === 'undefined') {
    return typeof module.exports === 'object'
      ? Object.assign(module.exports, { default: module.exports })
      : { default: module.exports };
  }

  return module.exports;
}
