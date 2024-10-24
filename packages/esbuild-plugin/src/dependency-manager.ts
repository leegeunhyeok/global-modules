import type { DependencyGraph, Module } from 'esbuild-dependency-graph';

export interface DependencyManager
  extends Pick<
    InstanceType<typeof DependencyGraph>,
    | 'hasModule'
    | 'getModule'
    | 'addModule'
    | 'updateModule'
    | 'removeModule'
    | 'dependenciesOf'
    | 'dependentsOf'
    | 'inverseDependenciesOf'
  > {
  /**
   * Register module to registry.
   */
  register: (actualPath: string) => Module;
  /**
   * Updates the dependency graph based on the module file at the actual path.
   *
   * Usage: trigger when the module file has changed.
   */
  syncModule: (actualPath: string) => Promise<Module>;
}
