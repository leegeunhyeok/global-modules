import fs from 'node:fs/promises';
import path from 'node:path';
import esbuild from 'esbuild';

const rootDir = path.resolve(import.meta.dirname, './src');
const outdir = path.resolve(import.meta.dirname, './dist');

const result = await esbuild.build({
  bundle: true,
  entryPoints: [path.join(rootDir, 'index.ts')],
  outdir: path.resolve(import.meta.dirname, './dist'),
  metafile: true,
});

await fs.writeFile(
  path.join(outdir, 'metafile.json'),
  JSON.stringify(result.metafile, null, 2),
  'utf-8'
);
