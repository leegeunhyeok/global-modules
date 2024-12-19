import * as esresolve from 'esresolve';
import type { BuildOptions } from 'esbuild';
import { DependencyGraph, type Module } from 'esbuild-dependency-graph';
import type { DependencyManager } from './dependency-manager';

export interface ModuleRegistryOptions {
  /**
   * Root path for lookup modules.
   *
   * Defaults to `process.cwd()`.
   */
  root?: string;
}

export class ModuleRegistry
  extends DependencyGraph
  implements DependencyManager
{
  private buildOptions?: BuildOptions;
  private root: string;

  constructor(options?: ModuleRegistryOptions) {
    const completeOptions: Required<ModuleRegistryOptions> = {
      root: process.cwd(),
      ...options,
    };
    super(completeOptions);
    this.root = completeOptions.root;
  }

  setBuildOptions(buildOptions: BuildOptions): void {
    this.buildOptions = buildOptions;
  }

  register(actualPath: string): Module {
    return this.hasModule(actualPath)
      ? this.getModule(actualPath)
      : this.addModule(actualPath, { dependents: [], dependencies: [] });
  }

  async syncModule(actualPath: string): Promise<Module> {
    const resolveResult = await esresolve.resolveFrom(actualPath, {
      root: this.root,
      alias: this.buildOptions?.alias,
      conditionNames: this.buildOptions?.conditions,
      mainFields: this.buildOptions?.mainFields,
      extensions: this.buildOptions?.resolveExtensions,
    });

    // Register or update target module's dependencies first
    const dependencies = resolveResult.map(
      (result) => this.register(result.path).id,
    );

    const imports = resolveResult.reduce((prev, curr) => {
      return { ...prev, [curr.request]: this.getModule(curr.path).id };
    }, {});

    // And register or update target module.
    return this.hasModule(actualPath)
      ? this.updateModule(actualPath, {
          dependencies,
          dependents: this.getModule(actualPath).dependents,
          meta: { imports },
        })
      : this.addModule(actualPath, {
          dependencies,
          dependents: [],
          meta: { imports },
        });
  }
}
