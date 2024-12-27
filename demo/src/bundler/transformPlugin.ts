import * as path from 'node:path';
import * as fs from 'node:fs';
import { createPlugin } from '@global-modules/esbuild-plugin';
import { Phase } from '@global-modules/swc-plugin';
import { transform } from './transform';

export function createTransformPlugin() {
  return createPlugin({
    name: 'transform-plugin',
    setup(build) {
      build.onLoad({ filter: /\.(?:[mc]js|[tj]sx?)$/ }, async (args) => {
        const source = await fs.promises.readFile(args.path, {
          encoding: 'utf-8',
        });

        const code = await transform(source, path.basename(args.path), {
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
        });

        return { loader: 'js', contents: registerHotModule(code, args.id) };
      });
    },
  });
}

function registerHotModule(code: string, id: number) {
  return [
    code,
    `
    if (window.__hot) {
      const context = window.__hot.register(${id});
      context.dispose(() => console.log('[HMR] Disposed', ${id}));
      context.accept(() => console.log('[HMR] Accepted', ${id}));
    }
    `,
  ].join('\n');
}
