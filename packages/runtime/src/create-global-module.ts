import { createExports, isExports } from './exports';
import { interopDefaultExport } from './interop-default-export';
import type {
  Exports,
  GlobalModule,
  Module,
  ModuleContext,
  ModuleExports,
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

  function require(id: ModuleId): Exports {
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

  function createContext(): ModuleContext {
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
      reset: () => void (module.exports = createExports()),
    };
  }

  function context(id: ModuleId): ModuleContext {
    const module = {} as Module;

    module.id = id;
    module.context = createContext();
    moduleRegistry.set(id, module);

    return module.context;
  }

  function clear(): void {
    moduleRegistry.clear();
  }

  function getRegistry(): Map<ModuleId, Module> {
    return moduleRegistry;
  }

  return {
    context,
    require,
    import: (id) => Promise.resolve(require(id)),
    getRegistry,
    clear,
  };
}
