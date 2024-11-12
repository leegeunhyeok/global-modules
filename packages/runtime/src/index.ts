import type {
  DependencyMap,
  Exports,
  GlobalModuleRegistry,
  Module,
  ModuleExports,
  ModuleRequire,
  ModuleStatus,
} from './types';

// eslint-disable-next-line @typescript-eslint/no-explicit-any -- allow
type Global = any;

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
  globalContext?: Global;
}

// eslint-disable-next-line no-new-func, @typescript-eslint/no-implied-eval -- allow for get global context tricky.
const _global: Global = new Function('return this')();

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
  const moduleRegistry = new Map<ModuleId, Module>();

  // @internal
  function __get(id: ModuleId): Module {
    const module = moduleRegistry.get(id);

    if (module == null) {
      throw new Error(`module not found: ${String(id)}`);
    }

    return module;
  }

  // @internal
  function __exports(id: ModuleId, definitions: Record<string, unknown>): void {
    const module = __get(id);
    for (const key in definitions) {
      if (Object.prototype.hasOwnProperty.call(definitions, key)) {
        Object.defineProperty(module.exports, key, {
          enumerable: true,
          get: () => definitions[key],
        });
      }
    }
  }

  // @internal
  function __require(id: ModuleId, source: string): Exports {
    const dependency = __get(id).deps[source];

    switch (typeof dependency) {
      case 'number':
        return moduleRegistry.get(dependency)?.exports ?? {};

      case 'function':
        return dependency();

      default:
        throw new Error('invalid dependency');
    }
  }

  function define(
    factory: (exports: ModuleExports, require: ModuleRequire) => void,
    id: ModuleId,
    deps: DependencyMap = {},
    evaluate = true,
  ): void {
    const module = factory as Module;

    module.id = id;
    module.exports = {};
    module.deps = deps;
    module.status = 'idle';

    moduleRegistry.set(id, module);

    if (evaluate) {
      // eslint-disable-next-line no-useless-call -- evaluate module
      module.call(null, __exports.bind(null, id), __require.bind(null, id));
      module.status = 'ready';
    }
  }

  function update(id: ModuleId, deps: DependencyMap, evaluate = true): void {
    const module = __get(id);

    module.deps = deps;
    module.exports = {};
    module.status = 'stale';

    if (evaluate) {
      // eslint-disable-next-line no-useless-call -- Create new exports object and re-evaluate the module.
      module.call(null, __exports.bind(null, id), __require.bind(null, id));
      module.status = 'ready';
    }
  }

  function status(id: ModuleId): ModuleStatus {
    const module = __get(id);

    return module.status;
  }

  function clear(): void {
    moduleRegistry.clear();
  }

  return { define, update, status, clear };
}

export type { GlobalModuleRegistry };
