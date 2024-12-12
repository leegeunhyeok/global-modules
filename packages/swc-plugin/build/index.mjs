import path from 'node:path';
import esbuild from 'esbuild';

const rootDir = path.resolve(import.meta.dirname, '..');

/**
 * @type {import('esbuild').BuildOptions}
 */
const baseBuildOptions = {
  bundle: true,
  entryPoints: [path.join(rootDir, 'index.ts')],
  platform: 'node',
};

await Promise.all([
  esbuild.build({
    ...baseBuildOptions,
    format: 'cjs',
    outdir: path.resolve(import.meta.dirname, '../dist'),
  }),
  esbuild.build({
    ...baseBuildOptions,
    format: 'cjs',
    outExtension: {
      '.js': '.cjs',
    },
    outdir: path.resolve(import.meta.dirname, '../cjs'),
  }),
  esbuild.build({
    ...baseBuildOptions,
    entryPoints: [path.join(rootDir, 'index.mts')],
    format: 'esm',
    outExtension: {
      '.js': '.mjs',
    },
    outdir: path.resolve(import.meta.dirname, '../esm'),
  }),
]);
