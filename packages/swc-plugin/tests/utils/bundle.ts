import assert from 'node:assert';
import path from 'node:path';
import * as swc from '@swc/core';
import * as esbuild from 'esbuild';
import { Phase } from '../../types';

interface BundleInput {
  entry: string;
  foo: string;
}

export async function bundle(input: BundleInput, { phase }: { phase: Phase }) {
  const { code: fooCode } = await swc.transformSync(input.foo, {
    jsc: {
      target: 'es5',
      experimental: {
        plugins: [
          [
            path.resolve(
              import.meta.dirname,
              '../../swc_plugin_global_modules.wasm',
            ),
            { id: '0', phase },
          ],
        ],
      },
    },
  });

  const buildResult = await esbuild.build({
    absWorkingDir: '/',
    bundle: true,
    write: false,
    format: 'iife',
    stdin: {
      contents: input.entry,
      loader: 'js',
    },
    plugins: [
      {
        name: 'foo-linker-plugin',
        setup(build) {
          build.onResolve({ filter: /^\.\/foo/ }, () => ({
            path: 'foo',
            namespace: 'foo-linker',
          }));

          build.onLoad({ filter: /.*/, namespace: 'foo-linker' }, () => ({
            contents: fooCode,
            loader: 'js',
          }));
        },
      },
    ],
  });

  const bundleCode = buildResult.outputFiles[0]?.text;
  assert(bundleCode, 'invalid bundle result');

  return bundleCode;
}
