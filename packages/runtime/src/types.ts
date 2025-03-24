// oxlint-disable-next-lineno-explicit-any -- allow
export type Global = any;

export type ModuleId = string;

export interface Module {
  id: ModuleId;
  context: ModuleContext;
}

export interface ModuleContext {
  exports: ModuleExports;
  module: {
    exports: Exports;
  };
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
   * Register new module to the global registry.
   */
  register: (id: ModuleId) => ModuleContext;
  /**
   * Get module exports from global registry.
   */
  require: ModuleRequire;
  /**
   * Get module exports from global registry (promise).
   */
  import: ModuleImport;
  /**
   * Get module from global registry.
   */
  getModule: (id: ModuleId) => Module;
  /**
   * Get global module registry.
   */
  getRegistry: () => Map<ModuleId, Module>;
  /**
   * Clear all modules from the registry.
   */
  clear: () => void;
}
