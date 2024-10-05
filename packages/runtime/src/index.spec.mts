import vm from 'node:vm';
import fs from 'node:fs/promises';
import path from 'node:path';
import { describe, it, expect, beforeAll, vi, Mock } from 'vitest';
import { DependencyGraph, isInternal } from 'esbuild-dependency-graph';
import { setup } from './index.js';

describe('@global-modules/runtime', () => {
  const REGISTRY_NAME = '__modules';
  const GLOBAL_CONTEXT_NAME = 'testGlobal';

  function createSandboxContext(sandbox?: vm.Context): SandboxContext {
    const context = vm.createContext({
      ...sandbox,
      setup,
      [GLOBAL_CONTEXT_NAME]: {},
    });

    function evaluate(code: string) {
      return new vm.Script(code).runInContext(context);
    }

    return {
      setup: () => {
        return evaluate(
          `setup({ registryName: ${JSON.stringify(
            REGISTRY_NAME
          )}, globalContext: ${GLOBAL_CONTEXT_NAME} })`
        );
      },
      getGlobalModuleRegistry: () => {
        return evaluate(`${GLOBAL_CONTEXT_NAME}.__modules`);
      },
      evaluate,
    };
  }

  interface SandboxContext {
    setup: () => any;
    getGlobalModuleRegistry: () => GlobalModuleRegistry;
    evaluate: (code: string) => any;
  }

  describe('when call `setup()` once', () => {
    let context: SandboxContext;

    beforeAll(() => {
      context = createSandboxContext();
    });

    it('should define module registry into global context', () => {
      // Before setup
      expect(context.getGlobalModuleRegistry()).toBeUndefined();

      context.setup();

      // After setup
      const globalRegistry = context.getGlobalModuleRegistry();
      expect(globalRegistry).toBeTruthy();
      expect(typeof globalRegistry.define).toEqual('function');
      expect(typeof globalRegistry.update).toEqual('function');
    });

    describe('when call `setup()` twice', () => {
      it('should throw error', () => {
        expect(() => context.setup()).toThrowError();
      });
    });
  });

  describe('when define a module to global registry', () => {
    let context: SandboxContext;
    let mockedLog: Mock;

    beforeAll(() => {
      mockedLog = vi.fn();

      context = createSandboxContext();
      context.setup();
      context.getGlobalModuleRegistry().define(() => {
        mockedLog('hello, world');
      }, 0);
    });

    it('should evaluate module immediately', () => {
      expect(mockedLog).toBeCalledTimes(1);
      expect(mockedLog).toBeCalledWith('hello, world');
    });
  });

  describe('when define modules to global registry', () => {
    const DEPENDENCY_IDS = {
      'src/__fixtures__/src/d.ts': 4,
      'src/__fixtures__/src/c.ts': 3,
      'src/__fixtures__/src/b.ts': 2,
      'src/__fixtures__/src/a.ts': 1,
      'src/__fixtures__/src/index.ts': 0,
    } as const;

    let context: SandboxContext;
    let mockedPrint: Mock;

    /**
     * Demo dependency graph
     *
     * ```
     * Entry (1)
     * +-- a (2)
     * +-- b (3)
     *     +-- c (4)
     *         +-- d (5)
     * ```
     */
    beforeAll(async () => {
      mockedPrint = vi.fn();

      const fixtureScript = (
        await fs.readFile(
          path.resolve(import.meta.dirname, '__fixtures__/dist/index.js'),
          'utf-8'
        )
      ).replaceAll('$$GLOBAL_CONTEXT', GLOBAL_CONTEXT_NAME);

      context = createSandboxContext({ print: mockedPrint });
      context.setup();
      context.evaluate(fixtureScript);
    });

    it('should evaluate module immediately', () => {
      expect(mockedPrint).toBeCalledTimes(1);
      expect(mockedPrint).toBeCalledWith(10, 90);
    });

    describe('when re-define specified module and update its reverse dependencies', () => {
      beforeAll(async () => {
        const metafile = await fs.readFile(
          path.resolve(import.meta.dirname, '__fixtures__/dist/metafile.json'),
          'utf-8'
        );
        const graph = new DependencyGraph(metafile);

        const { define, update } = context.getGlobalModuleRegistry();

        // Module: d
        const targetModule = 'src/__fixtures__/src/d.ts';
        define((_$import, $exports) => {
          $exports.d = 100; // Change exports value (Before: 40)
        }, DEPENDENCY_IDS[targetModule]);

        const inverseDependencies = graph
          .inverseDependenciesOf(targetModule)
          .filter((path) => path !== targetModule)
          .map((path) => {
            const module = graph.getModule(path);

            return {
              id: DEPENDENCY_IDS[path],
              inverseDependencies: isInternal(module)
                ? module.imports.map(({ path }) => DEPENDENCY_IDS[path])
                : [],
            };
          });

        // Update reverse dependencies (parents)
        inverseDependencies.forEach(({ id, inverseDependencies }) => {
          update(id, inverseDependencies);
        });
      });

      it('should re-evaluate each modules', () => {
        // 2nd call
        expect(mockedPrint).toBeCalledTimes(2);
        expect(mockedPrint).toBeCalledWith(10, 150);
      });
    });
  });
});
