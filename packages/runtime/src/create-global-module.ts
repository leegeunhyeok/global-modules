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
    dependencies: (() => unknown)[] | null = null,
  ): void {
    const module = moduleRegistry.has(id) ? getModule(id) : {} as Module;

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
      const boundRequire = require.bind(
        dependencyMap,
      ) as ModuleContext['require'];

      module.context.require = boundRequire;
      module.context.import = utils.toImport(boundRequire);
    }

    module.factory(module.context);
  }

  function require(
    this: null | Record<string, string> | (() => Exports)[],
    id: ModuleId,
    dependencyIndex?: number,
  ): Exports {
    if (Array.isArray(this) && typeof dependencyIndex === 'number') {
      // Bundle phase
      // Get module exports from the dependency getter.
      return this[dependencyIndex]();
    }

    if (this !== null && typeof this[id] === 'string') {
      // Runtime phase (`apply` called with dependency id map)
      // Remap the dependency id to the provided module id.
      id = this[id];
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
    dependencies: (() => unknown)[] | null,
  ): ModuleContext {
    const module = { exports: createExports() };
    const boundRequire = require.bind(dependencies) as ModuleContext['require'];

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
      require: boundRequire,
      import: utils.toImport(boundRequire),
    };
  }

  function clear(): void {
    moduleRegistry.clear();
  }

  function getRegistry(): Map<ModuleId, Module> {
    return moduleRegistry;
  }

  return { define, apply, require, clear, getModule, getRegistry };
}
