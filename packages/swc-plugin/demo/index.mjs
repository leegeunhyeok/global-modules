import * as fs from 'node:fs';
import * as path from 'node:path';
import * as swc from '@swc/core';
import globalModulePlugin, { Phase } from '../esm/index.mjs';

const code = await fs.promises.readFile(
  path.join(import.meta.dirname, 'code.js'),
  {
    encoding: 'utf-8',
  },
);

function registerPreset() {
  return [
    globalModulePlugin,
    {
      id: 1,
      phase: Phase.Register,
    },
  ];
}

function runtimePreset() {
  return [
    globalModulePlugin,
    {
      id: 1,
      phase: Phase.Runtime,
      dependencies: {
        react: 1000,
        './foo': 1001,
        './bar': 1002,
        './baz': 1003,
        './Component': 1004,
        './cjs-1': 1005,
        './cjs-2': 1006,
        './cjs-3': 1007,
        './esm': 1008,
        './re-exp': 1009,
        './re-exp-1': 1010,
        './re-exp-2': 1011,
        './re-exp-3': 1012,
        './re-exp-4': 1013,
        './re-exp-5': 1014,
      },
    },
  ];
}

// Register
console.log(
  (
    await swc.transform(code, {
      filename: 'test.js',
      jsc: {
        target: 'esnext',
        experimental: {
          plugins: [registerPreset()],
        },
      },
    })
  ).code,
);

// Runtime
console.log(
  (
    await swc.transform(code, {
      filename: 'test.js',
      jsc: {
        target: 'esnext',
        experimental: {
          plugins: [runtimePreset()],
        },
      },
    })
  ).code,
);
