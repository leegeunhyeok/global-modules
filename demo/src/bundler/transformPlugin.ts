import { createPlugin } from '@global-modules/esbuild-plugin';
import { Phase } from '@global-modules/swc-plugin';
import { transform } from './transform';

export function createTransformPlugin() {
  return createPlugin({
    name: 'transform-plugin',
    setup(build) {
      build.onLoad({ filter: /\.(?:[mc]js|[tj]sx?)$/ }, async (args) => {
        const code = await transform(args, {
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

        return { loader: 'js', contents: code };
      });
    },
  });
}
