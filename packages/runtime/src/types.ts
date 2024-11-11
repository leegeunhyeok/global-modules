export interface Module {
  (exports: ModuleExports, require: ModuleRequire): void;
  id: ModuleId;
  exports: Exports;
  deps: DependencyMap;
  status: ModuleStatus;
}

export type ModuleStatus = 'idle' | 'stale' | 'ready';
export type ModuleExports = (definitions: Exports) => void;
export type ModuleRequire = (source: string) => ModuleExports;
export type Exports = Record<string, unknown>;

export type DependencyMap = Record<
  string,
  ModuleId | Exports | (() => Exports)
>;

export interface GlobalModuleRegistry {
  /**
   * Defines a global module.
   *
   * This function can handle two cases:
   * - Case 1: When defining the initial module.
   *   - Dependencies must be passed as an object(ESM) or a function(CommonJS).
   *   - Directly references the module object.
   * - Case 2: When redefining an existing module. (eg. HMR)
   *   - The ID of the defined module must be passed.
   *   - References the module corresponding to the ID.
   *
   * **Case 1**
   *
   * ```ts
   * define((exports, require) => {
   *   const mod0 = require('./foo');
   *   const mod1 = require('./bar');
   *   const mod2 = require('./baz');
   *
   *   // module body
   *
   *   exports({
   *     named: expr_1,
   *     default: expr_2,
   *   });
   * }, 1000, {
   *   './foo': foo,
   *   './bar': bar,
   *   './baz': () => baz, // when CommonJS or Dynamic imports (for lazy evaluation)
   * });
   * ```
   *
   * ---
   *
   * **Case 2**
   *
   * ```ts
   * define(..., 1001, {}); // foo
   * define(..., 1002, {}); // bar
   * define(..., 1003, {}); // baz
   *
   * define((exports, require) => {
   *   const mod0 = require('./foo');
   *   const mod1 = require('./bar');
   *   const mod2 = require('./baz');
   *
   *   // module body
   *
   *   exports({
   *     named: expr_1,
   *     default: expr_2,
   *   });
   * }, 1000, { './foo': 1001, './bar': 1002, './baz': 1003 });
   * ```
   *
   */
  define: (
    factory: (exports: ModuleExports, require: ModuleRequire) => void,
    id: ModuleId,
    deps?: DependencyMap,
    evaluate?: boolean,
  ) => void;
  /**
   * Re-evaluates the specified defined module and creates a new exports object.
   *
   * It also propagates the changes to ensure that inverse dependency modules(parents) reference the new exports object.
   *
   * ```ts
   * // Re-evaluates `1001` module with reference dependencies via id.
   * update(1010, { './foo': 1007, './bar': 1004, './baz': 1003 });
   * ```
   */
  update: (id: ModuleId, deps?: DependencyMap, evaluate?: boolean) => void;
  /**
   * Get module's status.
   */
  status: (id: ModuleId) => ModuleStatus;
  /**
   * Clear all modules from the registry.
   */
  clear: () => void;
}
