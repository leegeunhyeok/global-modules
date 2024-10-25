export interface Module {
  (exports: ModuleExports, require: ModuleRequire): void;
  id: ModuleId;
  exports: ModuleExports;
  deps: DependencyMap;
  status: ModuleStatus;
}

export type ModuleStatus = 'idle' | 'stale' | 'ready';
export type ModuleExports = Record<string, unknown>;
export type ModuleRequire = (index: number) => ModuleExports;

export type DependencyMap = (
  | ModuleId
  | ModuleExports
  | (() => ModuleExports)
)[];

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
   *   const mod0 = require(0); // foo
   *   const mod1 = require(1); // bar
   *   const mod2 = require(2); // baz
   *
   *   // module body
   *
   *   exports.named = expr_1;
   *   exports.default = expr_2;
   * }, 1000, [
   *   foo,
   *   bar,
   *   () => baz, // when CommonJS (for lazy evaluation)
   * ]);
   * ```
   *
   * ---
   *
   * **Case 2**
   *
   * ```ts
   * define(..., 1001, []); // foo
   * define(..., 1002, []); // bar
   * define(..., 1003, []); // baz
   *
   * define((exports, require) => {
   *   const mod0 = require(0); // foo
   *   const mod1 = require(1); // bar
   *   const mod2 = require(2); // baz
   *
   *   // module body
   *
   *   exports.named = expr_1;
   *   exports.default = expr_2;
   * }, 1000, [1001, 1002, 1003]);
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
   * // Re-evaluates `1001` module and propagates the changes to `1007`, `1004`, `1003` modules.
   * update(1010, [1007, 1004, 1003]);
   * ```
   */
  update: (
    id: ModuleId,
    inverseDependencies?: ModuleId[],
    evaluate?: boolean,
  ) => void;
  /**
   * Get module's status.
   */
  status: (id: ModuleId) => ModuleStatus;
  /**
   * Clear all modules from the registry.
   */
  clear: () => void;
}
