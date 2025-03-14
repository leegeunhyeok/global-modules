// oxlint-disable-next-lineno-explicit-any -- allow
export type Global = any;

export type ModuleId = string;

export interface Module {
  id: ModuleId;
  context: ModuleContext;
  factory: ModuleFactory;
}

export type ModuleFactory = (context: ModuleContext) => void;

export interface ModuleContext {
  exports: ModuleExports;
  module: {
    exports: Exports;
  };
  require: ModuleRequire;
  import: ModuleImport;
}

export interface ModuleExports {
  (definitions: () => Exports): void;
  ns: (exports: Exports) => Exports;
}
export type ModuleRequire = (id: ModuleId) => Exports;
export type ModuleImport = (id: ModuleId) => Promise<Exports>;
export type Exports = Record<string, unknown>;

export interface GlobalModule {
  /**
   * Define a new module to the global registry.
   */
  define: (
    moduleFactory: ModuleFactory,
    id: ModuleId,
    dependencies?: Record<string, unknown> | null,
  ) => void;
  /**
   * Re-evaluate the module with the provided dependency id map.
   */
  apply: (id: ModuleId, dependencyMap?: Record<string, string>) => void;
  /**
   * Get global module registry.
   */
  getRegistry: () => Map<ModuleId, Module>;
  /**
   * Get module exports from global registry.
   */
  require: (id: ModuleId) => Exports;
  /**
   * Clear all modules from the registry.
   */
  clear: () => void;
}
