import type {
  Exports,
  GlobalModule,
  Module,
  ModuleContext,
  ModuleExports,
} from './types';
import * as utils from './utils';

export function createGlobalModule(): GlobalModule {
  const moduleRegistry = new Map<ModuleId, Module>();

  function ExportObject(): void {}

  function __get(id: ModuleId): Module {
    const module = moduleRegistry.get(id);

    if (module == null) {
      throw new Error(`module not found: ${String(id)}`);
    }

    return module;
  }

  function __exportsObject(): Exports {
    return new ExportObject() as Exports;
  }

  function __require(id: ModuleId): Exports {
    const module = __get(id);

    return module.exports instanceof ExportObject
      ? module.exports
      : { default: module.exports };
  }

  function __exports(
    exports: Exports,
    definitions: () => Record<string, unknown>,
  ): void {
    utils.__copyProps(exports, definitions());
  }

  function __ns(exports: Exports): Exports {
    const nsExports = __exportsObject();

    // In the case of namespace exports (re-export all), the `default` field must be excluded.
    utils.__copyProps(nsExports, exports, 'default');

    return nsExports;
  }

  function __createContext(module: Module): ModuleContext {
    const require = __require;
    const exports = Object.assign(
      ((definitions) => {
        __exports(module.exports, definitions);
      }) as ModuleExports,
      { ns: __ns },
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
    module.exports = __exportsObject();
    module.context = __createContext(module);
    moduleRegistry.set(id, module);

    return module.context;
  }

  function getContext(id: ModuleId): ModuleContext {
    const module = __get(id);

    // Update to new exports object.
    module.exports = __exportsObject();

    return module.context;
  }

  function clear(): void {
    moduleRegistry.clear();
  }

  return { register, getContext, clear };
}
