import { createExports, isExports } from './exports';
import { interopDefaultExport } from './interop-default-export';
import type {
  Exports,
  GlobalModule,
  Module,
  ModuleContext,
  ModuleExports,
  ModuleFactory,
  ModuleId,
} from './types';
import * as utils from './utils';

export function createGlobalModule(): GlobalModule {
  const moduleRegistry = new Map<ModuleId, Module>();

  function __exports(
    exports: Exports,
    definitions: () => Record<string, unknown>,
  ): void {
    utils.copyProps(exports, definitions());
  }

  function define(
    moduleFactory: ModuleFactory,
    id: ModuleId,
    dependencies: Record<string, unknown> | null = null,
  ): void {
    const module = {} as Module;

    module.id = id;
    module.context = createContext(dependencies);
    module.factory = moduleFactory;

    // Register the module after the module is evaluated.
    module.factory(module.context);
    moduleRegistry.set(id, module);
  }

  function apply(id: ModuleId, dependencyMap?: Record<string, string>): void {
    const module = getModule(id);

    if (dependencyMap != null) {
      // Override the require function with the provided dependency id map.
      module.context.require = require.bind(dependencyMap);
    }

    module.factory(module.context);
  }

  let re = 0;
  function require(
    this: null | Record<string, (() => Exports) | string>,
    id: ModuleId,
  ): Exports {
    if (this !== null) {
      const dependency = this[id];

      if (typeof dependency === 'function') {
        // Bundle phase (dependency getter)
        return dependency();
      }

      if (typeof dependency === 'string') {
        // Runtime phase (`apply` called with dependency id map)
        // Remap the dependency id to the provided module id.
        id = dependency;
      }
    }

    const module = getModule(id).context.module;

    return module.exports.__esModule || isExports(module.exports)
      ? module.exports
      : interopDefaultExport(module);
  }

  function getModule(id: ModuleId): Module {
    const module = moduleRegistry.get(id);

    if (module == null) {
      throw new Error(`module not found: '${id}'`);
    }

    return module;
  }

  function toNamespaceExports(exports: Exports): Exports {
    const nsExports = createExports();

    // In the case of namespace exports (re-export all), the `default` field must be excluded.
    utils.copyProps(nsExports, exports, 'default');

    return nsExports;
  }

  function createContext(
    dependencies: Record<string, unknown> | null,
  ): ModuleContext {
    const module = { exports: createExports() };

    return {
      // Exports object
      module,
      // Exports function
      //
      // `context.exports(...);`
      // `context.exports.ns(...);`
      exports: Object.assign(
        ((definitions) => {
          __exports(module.exports, definitions);
        }) as ModuleExports,
        { ns: toNamespaceExports },
      ),
      require: require.bind(dependencies),
    };
  }

  function clear(): void {
    moduleRegistry.clear();
  }

  function getRegistry(): Map<ModuleId, Module> {
    return moduleRegistry;
  }

  return { define, apply, require, clear, getRegistry };
}
