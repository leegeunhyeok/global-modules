interface RuntimeConfig {
  /**
   * Name of the module registry to be registered in the global object.
   *
   * Defaults to `__modules`.
   */
  registryName?: string;
  /**
   * The context to be used as the global object is provided.
   *
   * Defaults to `global object` of current runtime context.
   */
  globalContext?: any;
}

const _global = new Function('return this')();

export function setup({
  registryName = '__modules',
  globalContext = _global,
}: RuntimeConfig = {}): void {
  if (registryName in globalContext) {
    throw new Error('setup() should be called only once per runtime context');
  }

  Object.defineProperty(globalContext, registryName, {
    value: getGlobalModuleRegistry(),
  });
}

function getGlobalModuleRegistry(): GlobalModuleRegistry {
  const registry: Record<ModuleId, ModuleContext> = {};

  function createImport(id: ModuleId): ModuleImport {
    return (index: number) => {
      // d: dependencyMap
      const target = registry[id]?.d?.[index];

      // Case 1: When the module is newly updated at runtime (eg. HMR)
      if (typeof target === 'number') {
        // x: exports
        return registry[target].x;
      }

      if (target == null) {
        throw new Error(`module not found (id: ${id}, index: ${index})`);
      }

      // Case 2: When the defined module is evaluated for the first time.
      return typeof target === 'function'
        ? target() // CommonJS
        : target; // ESM
    };
  }

  function createExports(): ModuleExports {
    return Object.create(null);
  }

  return {
    define: (factory, id, dependencyMap) => {
      const $import = createImport(id);
      const $exports = createExports();

      registry[id] = {
        f: factory,
        i: $import,
        x: $exports,
        d: dependencyMap,
      };

      factory($import, $exports);
    },
    update: (id, dependencyIds) => {
      const targetModule = registry[id];

      if (targetModule == null) {
        throw new Error(`module not found (id: ${id})`);
      }

      dependencyIds.forEach((moduleId, index) => {
        // d: dependencyMap
        targetModule.d[index] = moduleId;
      });

      // => factory(import, exports);
      targetModule.f(targetModule.i, (targetModule.x = createExports()));
    },
  };
}
