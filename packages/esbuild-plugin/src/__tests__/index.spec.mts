import assert from 'node:assert';
import vm from 'node:vm';
import fs from 'node:fs/promises';
import path from 'node:path';
import * as esbuild from 'esbuild';
import { vi, describe, it, expect, beforeAll } from 'vitest';
import { createPlugin, type CreatePluginResult } from '../create-plugin.js';
import {
  generateInitialModule,
  generateUpdatedModule,
} from './new-module-helper.mjs';

type RuntimeResult = unknown;

const ROOT = import.meta.dirname;
const FIXTURE_ROOT = path.join(ROOT, '__fixtures__');

describe('@global-modules/esbuild-plugin', () => {
  let buildResult: esbuild.BuildResult;
  let createPluginResult: CreatePluginResult;
  const mockedPrint = vi.fn();
  const mockedRegister = vi.fn();

  function createSandboxContext(sandbox?: vm.Context): SandboxContext {
    const context = vm.createContext(sandbox);

    function evaluate(code: string): RuntimeResult {
      return new vm.Script(code).runInContext(context);
    }

    return { evaluate };
  }

  interface SandboxContext {
    evaluate: (code: string) => RuntimeResult;
  }

  beforeAll(async () => {
    createPluginResult = createPlugin({
      name: 'test-transform-plugin',
      setup(build) {
        build.onLoad({ filter: /.*/ }, async (args) => {
          const rawCode = await fs.readFile(args.path, 'utf-8');

          // `args.id` must exist.
          assert(typeof args.id === 'number', '`id` is not exist');

          return {
            loader: 'ts',
            contents: [
              // Original code
              rawCode,
              // Inject `register` function call expression into module body.
              `register(${String(args.id)});`,
            ].join('\n'),
          };
        });
      },
    });

    buildResult = await esbuild.build({
      entryPoints: [path.join(FIXTURE_ROOT, 'index.ts')],
      write: false,
      bundle: true,
      logLevel: 'silent',
      plugins: [createPluginResult.plugin],
    });
  });

  it('should an extended args object must be provided', () => {
    // The plugin has assertions for `args.id`.
    expect(buildResult).toBeTruthy();
    expect(buildResult.errors).toHaveLength(0);
  });

  it('should the `register` function added by the plugin be called', () => {
    const context = createSandboxContext({
      print: mockedPrint,
      register: mockedRegister,
    });

    // Evaluate bundle from sandbox context.
    context.evaluate(
      // oxlint-disable-next-line no-non-null-assertion
      Buffer.from(buildResult.outputFiles![0].contents).toString(),
    );

    // `print` function call expression in `__fixtures__/index.ts`.
    expect(mockedPrint).toBeCalledTimes(1);
    expect(mockedPrint).toBeCalledWith(10, 90);

    // `register` function call expressions are in each modules.
    const moduleCount = Object.keys(buildResult.metafile?.inputs ?? {}).length;
    expect(mockedRegister).toBeCalledTimes(moduleCount);
  });

  describe('Dependency manager', () => {
    it('hasModule', () => {
      const { dependencyManager } = createPluginResult;

      expect(
        dependencyManager.hasModule('src/__tests__/__fixtures__/index.ts'),
      ).toBe(true);
      expect(dependencyManager.hasModule('not/exist/module.ts')).toBe(false);
    });

    it('getModule', () => {
      const { dependencyManager } = createPluginResult;

      const entryModule = dependencyManager.getModule(
        'src/__tests__/__fixtures__/index.ts',
      );

      // `src/__tests__/__fixtures__/a.ts`
      // `src/__tests__/__fixtures__/b.ts`
      expect(entryModule.dependencies).toHaveLength(2);
    });

    it('addModule', async () => {
      await generateInitialModule();

      const { dependencyManager } = createPluginResult;

      const module = dependencyManager.addModule(
        'src/__tests__/__fixtures__/new-module.ts',
      );

      expect(module.dependencies).toHaveLength(0);
    });

    it('updateModule', () => {
      const { dependencyManager } = createPluginResult;

      const module = dependencyManager.updateModule(
        'src/__tests__/__fixtures__/new-module.ts',
        [],
        ['src/__tests__/__fixtures__/index.ts'],
      );

      /**
       * Before update
       *
       * ```
       * new.ts (dependent count: 0)
       * ```
       *
       * ```
       * index.ts
       *   new.ts (dependent count: 1)
       * ```
       */
      expect(module.dependencies).toHaveLength(0);
      expect(module.dependents).toHaveLength(1);
    });

    it('syncModule', async () => {
      await generateInitialModule();

      const { dependencyManager } = createPluginResult;
      const modulePath = 'src/__tests__/__fixtures__/new-module.ts';

      let targetModule = await dependencyManager.syncModule(modulePath);

      /**
       * Before sync
       *
       * ```
       * new.ts (dependency count: 0)
       * ```
       *
       * After sync
       *
       * ```
       * new.ts (dependency count: 2)
       *   new-deps-1.ts
       *   new-deps-2.ts
       * ```
       */
      expect(targetModule.dependencies).toHaveLength(2);

      // Update imports
      await generateUpdatedModule();

      targetModule = await dependencyManager.syncModule(
        'src/__tests__/__fixtures__/new-module.ts',
      );

      /**
       * Before sync
       *
       * ```
       * new.ts (dependency count: 2)
       *   new-deps-1.ts
       *   new-deps-2.ts
       * ```
       *
       * After sync
       * ```
       * new.ts (dependency count: 3)
       *   new-deps-1.ts
       *   new-deps-2.ts
       *   new-deps-3.ts
       * ```
       */
      expect(targetModule.dependencies).toHaveLength(3);
    });

    it('removeModule', () => {
      const { dependencyManager } = createPluginResult;

      // Remove `c.ts` from registry.
      dependencyManager.removeModule('src/__tests__/__fixtures__/c.ts');
      const module = dependencyManager.getModule(
        'src/__tests__/__fixtures__/b.ts',
      );

      /**
       * Before remove
       *
       * ```
       * index.ts
       *   b.ts (dependency count: 1)
       *     c.ts
       * ```
       *
       * After remove
       *
       * ```
       * index.ts
       *   b.ts (dependency count: 0)
       * ```
       */
      expect(module.dependencies).toHaveLength(0);
    });
  });
});
