import { createExports, isExports } from './exports';
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

  function __require(id: ModuleId): Exports {
    const module = getModule(id);

    return module.exports.__esModule || isExports(module.exports)
      ? module.exports
      : { default: module.exports };
  }

  function __exports(
    exports: Exports,
    definitions: () => Record<string, unknown>,
  ): void {
    utils.copyProps(exports, definitions());
  }

  function getModule(id: ModuleId): Module {
    const module = moduleRegistry.get(id);

    if (module == null) {
      throw new Error(`module not found: ${String(id)}`);
    }

    return module;
  }

  function toNamespaceExports(exports: Exports): Exports {
    const nsExports = createExports();

    // In the case of namespace exports (re-export all), the `default` field must be excluded.
    utils.copyProps(nsExports, exports, 'default');

    return nsExports;
  }

  function createContext(module: Module): ModuleContext {
    const require = __require;
    const exports = Object.assign(
      ((definitions) => {
        __exports(module.exports, definitions);
      }) as ModuleExports,
      { ns: toNamespaceExports },
    );

    return {
      require,
      exports,
      module,
    };
  }

  function register(id: ModuleId): ModuleContext {
    const module = {} as Module;

    module.id = id;
    module.exports = createExports();
    module.context = createContext(module);
    moduleRegistry.set(id, module);

    return module.context;
  }

  function getContext(id: ModuleId): ModuleContext {
    const module = getModule(id);

    // Update to new exports object.
    module.exports = createExports();

    return module.context;
  }

  function clear(): void {
    moduleRegistry.clear();
  }

  return { register, getContext, clear };
}
