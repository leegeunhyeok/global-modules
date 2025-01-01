// oxlint-disable-next-lineno-explicit-any -- allow
export type Global = any;

export type ModuleId = number;

export interface Module {
  id: ModuleId;
  exports: Exports;
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
export type Exports = Record<string, unknown>;

export interface GlobalModule {
  /**
   * Register new module to the global registry.
   */
  register: (id: ModuleId) => ModuleContext;
  /**
   * Get module context from global registry.
   */
  getContext: (id: ModuleId) => ModuleContext;
  /**
   * Get module exports from global registry.
   */
  require: (id: ModuleId) => Exports;
  /**
   * Clear all modules from the registry.
   */
  clear: () => void;
}
