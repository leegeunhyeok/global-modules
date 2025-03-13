import assert from 'node:assert';
import fs from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';
import * as swc from '@swc/core';
import * as esbuild from 'esbuild';

const require = createRequire(import.meta.url);
const runtimePath = require.resolve('@global-modules/runtime');
const runtimeCode = await fs.readFile(runtimePath, 'utf-8');

export interface BundleInput {
  [file: string]: string;
}

export async function bundleWithFoo(
  input: {
    entry: string;
    foo: string;
  },
  { runtime }: { runtime: boolean },
) {
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
            { id: '0', runtime },
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
    logLevel: 'error',
    stdin: {
      contents: input.entry,
      loader: 'js',
    },
    banner: {
      js: [
        runtimeCode,
        runtime ? `global.__modules.register('0');` : null,
      ]
        .filter(Boolean)
        .join('\n'),
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

export async function bundle(
  input: BundleInput,
  { index, runtime }: { index: number; runtime: boolean },
) {
  let entryFileName = '';
  const tempDir = path.join(import.meta.dirname, `./.temp/${String(index)}`);

  await fs
    .access(tempDir)
    .then(() => fs.rmdir(tempDir))
    .catch(() => void 0)
    .then(() => fs.mkdir(tempDir, { recursive: true }));

  await Promise.all(
    Object.entries(input).map(async ([file, code]) => {
      if (file.startsWith('entry')) {
        entryFileName = file;
      }

      const filePath = path.join(tempDir, file);
      await fs.writeFile(filePath, code);
    }),
  );

  const buildResult = await esbuild.build({
    entryPoints: [entryFileName],
    absWorkingDir: tempDir,
    bundle: true,
    write: false,
    format: 'iife',
    logLevel: 'error',
    banner: {
      js: runtimeCode,
    },
    plugins: [
      {
        name: 'transform-plugin',
        setup(build) {
          let id = 0;
          const moduleMap = new Map<string, number>();

          build.onLoad({ filter: /.*/ }, async (args) => {
            let currentId: number;

            if (moduleMap.has(args.path)) {
              currentId = moduleMap.get(args.path) as number;
            } else {
              currentId = id++;
              moduleMap.set(args.path, currentId);
            }

            const code = await fs.readFile(args.path, 'utf-8');
            const { code: transformedCode } = await swc.transform(code, {
              jsc: {
                target: 'es5',
                experimental: {
                  plugins: [
                    [
                      path.resolve(
                        import.meta.dirname,
                        '../../swc_plugin_global_modules.wasm',
                      ),
                      { id: String(currentId), runtime },
                    ],
                  ],
                },
              },
            });

            return {
              contents: transformedCode,
              loader: 'js',
            };
          });
        },
      },
    ],
  });

  const bundleCode = buildResult.outputFiles[0]?.text;
  assert(bundleCode, 'invalid bundle result');

  await fs.writeFile(path.join(tempDir, '_bundle.js'), bundleCode, 'utf-8');

  return bundleCode;
}
