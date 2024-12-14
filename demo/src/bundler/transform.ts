import * as path from 'node:path';
import * as fs from 'node:fs';
import globalModulePlugin, { PluginConfig } from '@global-modules/swc-plugin';
import * as swc from '@swc/core';
import { OnLoadArgs } from 'esbuild';

export async function transform(args: OnLoadArgs, pluginConfig: PluginConfig) {
  const source = await fs.promises.readFile(args.path, {
    encoding: 'utf-8',
  });

  const { code } = await swc.transform(source, {
    filename: path.basename(args.path),
    configFile: false,
    jsc: {
      parser: {
        syntax: 'typescript',
        decorators: false,
        tsx: false,
      },
      target: 'es5',
      experimental: {
        plugins: [[globalModulePlugin, pluginConfig]],
      },
    },
  });

  return code;
}
