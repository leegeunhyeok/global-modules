import * as fs from 'node:fs';
import * as swc from '@swc/core';

export async function loadSource(path: string) {
  const source = await fs.promises.readFile(path, {
    encoding: 'utf-8',
  });

  const result = await swc.transform(source, {
    filename: path,
    sourceMaps: false,
    jsc: {
      target: 'es5',
    },
  });

  return result.code;
}
