import type {
  Exports,
  GlobalModuleRegistry,
  Module,
  ModuleContext,
  ModuleExports,
} from './types';

export function createGlobalModuleRegistry(): GlobalModuleRegistry {
  const moduleRegistry = new Map<ModuleId, Module>();

  function ExportObject(): void {}

  const __hasOwnProp = Object.prototype.hasOwnProperty;
  const __defProp = Object.defineProperty;
  const __copyProps = <T extends object>(
    destination: T,
    source: T,
    except?: string,
  ): void => {
    for (const key in source) {
      if (
        key !== except &&
        __hasOwnProp.call(source, key) &&
        !__hasOwnProp.call(destination, key)
      ) {
        __defProp(destination, key, {
          enumerable: true,
          get: () => source[key],
        });
      }
    }
  };

  function __get(id: ModuleId): Module {
    const module = moduleRegistry.get(id);

    if (module == null) {
      throw new Error(`module not found: ${String(id)}`);
    }

    return module;
  }

  function __exportObject(): Exports {
    return new ExportObject() as Exports;
  }

  function __exports(
    exports: Exports,
    definitions: () => Record<string, unknown>,
  ): void {
    __copyProps(exports, definitions());
  }

  function __require(id: ModuleId): Exports {
    const module = __get(id);

    return module.exports instanceof ExportObject
      ? module.exports
      : { default: module.exports };
  }

  function __ns(exports: Exports): Exports {
    const nsExports = __exportObject();

    // In the case of namespace exports (re-export all), the `default` field must be excluded.
    __copyProps(nsExports, exports, 'default');

    return nsExports;
  }

  function __createContext(module: Module): ModuleContext {
    const require = __require;
    const exports = Object.assign(
      ((definitions) => {
        return __exports(module.exports, definitions);
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
    module.exports = __exportObject();
    module.context = __createContext(module);
    moduleRegistry.set(id, module);

    return module.context;
  }

  function getContext(id: ModuleId): ModuleContext {
    const module = __get(id);

    module.exports = __exportObject();

    return module.context;
  }

  function clear(): void {
    moduleRegistry.clear();
  }

  return { register, getContext, clear };
}
