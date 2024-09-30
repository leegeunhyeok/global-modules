interface RuntimeConfig {
  /**
   * Name of the module registry to be registered in the global object.
   *
   * Defaults to `__modules`.
   */
  registryName?: string;
}

const global = new Function('return this')();

export function setup({
  registryName = '__modules',
}: RuntimeConfig = {}): void {
  if (registryName in global) {
    throw new Error('setup() should be called only once per runtime context');
  }

  Object.defineProperty(global, registryName, {
    value: getGlobalModuleRegistry(),
  });
}

function getGlobalModuleRegistry(): GlobalModuleRegistry {
  const registry: Record<ModuleId, ModuleContext> = {};

  function createImport(id: ModuleId): ModuleImport {
    return (source: string) => {
      const dependencyFactory = registry[id]?.$dependencyMap?.[source];

      if (dependencyFactory == null) {
        throw new Error(`'${source}' not found (id: ${id})`);
      }

      return dependencyFactory();
    };
  }

  function createExports(): ModuleExports {
    return Object.create(null);
  }

  return {
    define: (factory, id, dependencyMap = {}) => {
      const $import = createImport(id);
      const $exports = createExports();

      registry[id] = {
        factory,
        $import,
        $exports,
        $dependencyMap: dependencyMap,
      };

      factory($import, $exports);
    },
    update: (id, dependencyIds) => {
      const targetModule = registry[id];

      if (targetModule == null) {
        throw new Error(`module not found (id: ${id})`);
      }

      Object.entries(dependencyIds ?? {}).forEach(([source, moduleId]) => {
        targetModule.$dependencyMap[source] = () => registry[moduleId].$exports;
      });

      targetModule.factory(
        targetModule.$import,
        (targetModule.$exports = createExports())
      );
    },
  };
}
