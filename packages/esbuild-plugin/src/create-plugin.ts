import assert from 'node:assert';
import type { OnLoadArgs, OnLoadOptions, Plugin, PluginBuild } from 'esbuild';
import { ModuleRegistry, type ModuleRegistryOptions } from './module-registry';
import type { DependencyManager } from './dependency-manager';

type ModuleId = number;
type GlobalModuleOnLoad = Omit<PluginBuild, 'onLoad'> & {
  onLoad: (
    options: OnLoadOptions,
    callback: (
      args: OnLoadArgs & { id: ModuleId },
    ) => ReturnType<Parameters<PluginBuild['onLoad']>[1]>,
  ) => void;
};

interface GlobalModulePlugin extends Omit<Plugin, 'setup'> {
  setup: (
    build: Omit<PluginBuild, 'onLoad'> & GlobalModuleOnLoad,
  ) => void | Promise<void>;
}

export interface CreatePluginOptions {
  root?: string;
}

export interface CreatePluginResult {
  plugin: Plugin;
  dependencyManager: DependencyManager;
}

export function createPlugin(
  plugin: GlobalModulePlugin,
  options?: ModuleRegistryOptions,
): CreatePluginResult {
  const registry = new ModuleRegistry(options);

  const enhancedPlugin: Plugin = {
    name: plugin.name,
    setup(build) {
      // Force enable `metafile` option.
      build.initialOptions.metafile = true;

      // Set bundler's options
      build.onStart(() => {
        registry.setBuildOptions(build.initialOptions);
      });

      build.onEnd((result) => {
        assert(result.metafile, 'invalid metafile');

        registry.load(result.metafile);
      });

      return plugin.setup({
        ...build,
        // Override esbuild's `onLoad` handler for inject module ID.
        onLoad: (options, callback) => {
          build.onLoad(options, (args) =>
            callback({ ...args, id: registry.register(args.path).id }),
          );
        },
      });
    },
  };

  return { plugin: enhancedPlugin, dependencyManager: registry };
}
