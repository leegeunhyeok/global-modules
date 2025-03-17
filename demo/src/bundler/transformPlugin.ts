import * as path from 'node:path';
import * as fs from 'node:fs';
import { Plugin } from 'esbuild';
import { transform } from './transform';
import { registerHotModule } from './templates';

export interface TransformPluginOptions {
  resolveId: (id: string) => number;
}

export function createTransformPlugin(options: TransformPluginOptions): Plugin {
  return {
    name: 'transform-plugin',
    setup(build) {
      build.onLoad({ filter: /\.(?:[mc]js|[tj]sx?)$/ }, async (args) => {
        const source = await fs.promises.readFile(args.path, {
          encoding: 'utf-8',
        });

        const moduleId = options.resolveId(args.path).toString();
        const code = await transform(source, path.basename(args.path), {
          id: moduleId,
          // At the initial build, the bundling process should be
          // delegated to the bundler(eg. esbuild) using the `Phase.Bundle`.
          //
          // - `false` Bundle phase
          //   - Keep import & export statements.
          //   - Register module references using the Global Module API
          // - `true`: Runtime phase
          //   - Remove import & export statements.
          //   - Reference the module through the Global Module API.
          runtime: false,
        });

        return { loader: 'js', contents: registerHotModule(code, moduleId) };
      });
    },
  };
}
