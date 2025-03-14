import * as fs from 'node:fs';
import * as path from 'node:path';
import * as swc from '@swc/core';
import globalModulePlugin from '../esm/index.mjs';

const code = await fs.promises.readFile(
  path.join(import.meta.dirname, 'code.js'),
  {
    encoding: 'utf-8',
  },
);

function bundlePreset() {
  return [
    globalModulePlugin,
    {
      id: 'mod-id',
      runtime: false,
    },
  ];
}

function runtimePreset() {
  return [
    globalModulePlugin,
    {
      id: 'mod-id',
      runtime: true,
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
          plugins: [bundlePreset()],
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
