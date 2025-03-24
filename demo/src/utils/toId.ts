import type { Module } from 'esbuild-dependency-graph';

export function toId(module: Module) {
  return `${module.path}#${module.id}`;
}
