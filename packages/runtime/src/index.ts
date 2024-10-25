import type {
  DependencyMap,
  GlobalModuleRegistry,
  Module,
  ModuleExports,
  ModuleRequire,
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

  function require(moduleId: ModuleId, index: number): ModuleExports {
    const targetModule = moduleRegistry.get(moduleId);

    if (targetModule == null) {
      throw new Error(`module not found: ${String(moduleId)}`);
    }

    const dependency = targetModule.deps[index];

    switch (typeof dependency) {
      case 'number':
        return moduleRegistry.get(dependency)?.exports ?? {};

      case 'object':
        return dependency;

      case 'function':
        return dependency();

      default:
        throw new Error('invalid dependency');
    }
  }

  function define(
    factory: (exports: ModuleExports, require: ModuleRequire) => void,
    id: ModuleId,
    deps: DependencyMap = [],
  ): void {
    const module = factory as Module;

    module.id = id;
    module.exports = {};
    module.deps = deps;
    module.ready = false;

    moduleRegistry.set(id, module);

    // eslint-disable-next-line no-useless-call -- evaluate module
    module.call(null, module.exports, require.bind(null, id));
    module.ready = true;
  }

  function update(id: ModuleId, deps: ModuleId[]): void {
    const module = moduleRegistry.get(id);

    if (module == null) {
      throw new Error(`module not found: ${String(id)}`);
    }

    module.deps = deps;

    // eslint-disable-next-line no-useless-call -- Create new exports object and re-evaluate the module.
    module.call(null, (module.exports = {}), require.bind(null, id));
  }

  function clear(): void {
    moduleRegistry.clear();
  }

  return { define, update, clear };
}

export type { GlobalModuleRegistry };
