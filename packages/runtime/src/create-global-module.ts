import { createExports, isExports } from './exports';
import { interopDefaultExport } from './interopDefaultExport';
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
    const module = getModule(id);

    return module.exports.__esModule || isExports(module.exports)
      ? module.exports
      : interopDefaultExport(module);
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
    const exports = Object.assign(
      ((definitions) => {
        __exports(module.exports, definitions);
      }) as ModuleExports,
      { ns: toNamespaceExports },
    );

    return {
      exports,
      module,
      reset: () => void (module.exports = createExports()),
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
    return getModule(id).context;
  }

  function getRegistry(): Map<ModuleId, Module> {
    return moduleRegistry;
  }

  function clear(): void {
    moduleRegistry.clear();
  }

  return { register, getContext, getRegistry, require, clear };
}
