import * as swc from '@swc/core';
import { createPlugin } from '@global-modules/esbuild-plugin';
import * as fs from 'node:fs';
import * as path from 'node:path';
import globalModulePlugin, {
  Phase,
  type PluginConfig,
} from '@global-modules/swc-plugin';

export function createTransformPlugin() {
  return createPlugin({
    name: 'transform-plugin',
    setup(build) {
      build.onLoad({ filter: /\.(?:[mc]js|[tj]sx?)$/ }, async (args) => {
        const source = await fs.promises.readFile(args.path, {
          encoding: 'utf-8',
        });

        const pluginConfig: PluginConfig = {
          // Set the module ID provided by `@global-modules/esbuild-plugin`.
          id: args.id,
          // At the initial build, the bundling process should be
          // delegated to the bundler(eg. esbuild) using the Phase.Register.
          //
          // - Phase.Register
          //   - Keep import & export statements.
          //   - Register module references using the Global Module API
          // - Phase.Runtime
          //   - Remove import & export statements.
          //   - Reference the module through the Global Module API.
          phase: Phase.Register,
        };

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

        return { loader: 'js', contents: code };
      });
    },
  });
}
