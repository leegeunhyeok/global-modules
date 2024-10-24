# esbuild-plugin

An extension package for the esbuild plugin that expands global module ID values and helps manage dependencies.

## Features

- Automatically generates and registers module IDs.
- Provides a manager for dependency management.

## Usage

Inject customized runtime code using module IDs (e.g., for HMR).

```ts
import * as fs from 'node:fs/promises';
import * as esbuild from 'esbuild';
import * as swc from '@swc/core';
import { createPlugin } from '@global-modules/esbuild-plugin';

const { plugin, dependencyManager } = createPlugin({
  name: 'my-transform-plugin',
  setup(build) {
    // You can access the module ID from the `args` passed to the onLoad callback.
    build.onLoad({ filter: /.*/ }, async (args) => {
      const rawCode = fs.readFile(args.path, 'utf-8');

      return {
        loader: 'ts',
        // `args.id` is module's unique ID.
        contents: [rawCode, `global.hot.register(${args.id});`].join('\n'),
      };
    });
  },
});

await esbuild.build({
  /* Other build options */
  plugins: [plugin],
});

watcher.onChange(async (filePath) => {
  // Sync with actual module file.
  const updatedModule = await dependencyManager.syncModule(filePath);

  // Get inverse dependencies of updated module.
  const inverseDependencies = dependencyManager.inverseDependenciesOf(
    updatedModule.id,
  );

  // Transform the module for re-evaluation.
  const rawCode = await fs.readFile(filePath, 'utf-8');
  const result = await swc.transform(rawCode, {
    jsc: {
      experimental: {
        plugins: [
          [
            '@global-modules/swc-plugin',
            { dependencies: updatedModule.dependencies },
          ],
        ],
      },
    },
  });

  // Re-evaluation updated module and its inverse dependencies.
  context.hot.send([
    result.code,
    ...inverseDependencies.forEach(({ id, dependencies }) => {
      return `global.__modules.update(${id}, ${JSON.stringify(dependencies)})`;
    }),
  ]);
});
```

<details>

<summary>Types</summary>

```ts
interface DependencyManager
  extends Pick<
    InstanceType<typeof dependencyGraph.DependencyGraph>,
    | 'hasModule'
    | 'getModule'
    | 'addModule'
    | 'updateModule'
    | 'removeModule'
    | 'dependenciesOf'
    | 'dependentsOf'
    | 'inverseDependenciesOf'
  > {
  /**
   * Register module to registry.
   */
  register: (actualPath: string) => dependencyGraph.Module;
  /**
   * Updates the dependency graph based on the module file at the actual path.
   *
   * Usage: trigger when the module file has changed.
   */
  syncModule: (actualPath: string) => Promise<SyncedModuleData>;
}
```

</details>

The dependency graph is powered by [esbuild-dependency-graph](https://github.com/leegeunhyeok/esbuild-dependency-graph).

For more APIs of the dependency graph, check the respective repository.

## Related

- [@global-modules/runtime](https://github.com/leegeunhyeok/global-modules/tree/main/packages/runtime) by [@leegeunhyeok](https://github.com/leegeunhyeok)
- [@global-modules/swc-plugin](https://github.com/leegeunhyeok/global-modules/tree/main/packages/swc-plugin) by [@leegeunhyeok](https://github.com/leegeunhyeok)
- [esbuild-dependency-graph](https://github.com/leegeunhyeok/esbuild-dependency-graph) by [@leegeunhyeok](https://github.com/leegeunhyeok)
