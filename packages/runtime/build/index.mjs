import path from 'node:path';
import esbuild from 'esbuild';

const rootDir = path.resolve(import.meta.dirname, '../src');

await esbuild.build({
  bundle: true,
  entryPoints: [path.join(rootDir, 'index.ts')],
  format: 'iife',
  outdir: path.resolve(import.meta.dirname, '../dist'),
});
