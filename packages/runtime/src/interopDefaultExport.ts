import type { Module } from './types';

export function interopDefaultExport(module: Module) {
  if (typeof module.exports.default === 'undefined') {
    return typeof module.exports === 'object'
      ? Object.assign(module.exports, { default: module.exports })
      : { default: module.exports };
  }

  return module.exports;
}
