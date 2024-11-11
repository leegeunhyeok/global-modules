import vm from 'node:vm';
import fs from 'node:fs/promises';
import path from 'node:path';
import { describe, it, expect, beforeAll, vi, Mock } from 'vitest';
import { DependencyGraph } from 'esbuild-dependency-graph';
import { setup } from './index.js';
import { GlobalModuleRegistry } from './types.js';

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
            REGISTRY_NAME,
          )}, globalContext: ${GLOBAL_CONTEXT_NAME} })`,
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

  describe('when define multiple modules to global registry', () => {
    const DEPENDENCY_IDS = {
      'src/__fixtures__/src/index.ts': 0,
      'src/__fixtures__/src/a.ts': 1,
      'src/__fixtures__/src/b.ts': 2,
      'src/__fixtures__/src/c.ts': 3,
      'src/__fixtures__/src/d.ts': 4,
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
          'utf-8',
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
          'utf-8',
        );
        const graph = new DependencyGraph();

        // Add modules to ensure IDs order.
        Object.keys(DEPENDENCY_IDS).forEach((path) => graph.addModule(path));
        graph.load(metafile);

        const { define, update } = context.getGlobalModuleRegistry();

        // Module: d
        const targetModule = 'src/__fixtures__/src/d.ts';
        define((exports, _require) => {
          exports({ d: 100 }); // Change exports value (Before: 40)
        }, 4);

        function toOriginSource(modulePath: string): string {
          const basename = path.basename(modulePath);
          const extension = path.extname(basename);

          return './' + basename.replace(new RegExp(`${extension}$`), '');
        }

        const inverseDependenciesId = graph
          .inverseDependenciesOf(targetModule)
          .map((module) => ({
            id: module.id,
            dependencies: graph.dependenciesOf(module.id).reduce(
              (prev, module) => ({
                ...prev,
                [toOriginSource(module.path)]: module.id,
              }),
              {},
            ),
          }));

        // Update reverse dependencies (parents)
        inverseDependenciesId.forEach(({ id, dependencies }) => {
          update(id, dependencies);
        });
      });

      it('should re-evaluate each modules', () => {
        // 2nd call
        expect(mockedPrint).toBeCalledTimes(2);
        expect(mockedPrint).toBeCalledWith(10, 150);
      });
    });
  });

  describe('module status', () => {
    const MODULE_IDS = {
      foo: 0,
      bar: 1,
    } as const;
    let context: SandboxContext;

    beforeAll(async () => {
      context = createSandboxContext();
      context.setup();
    });

    describe('when the module has been evaluated', () => {
      it('should returns `ready`', () => {
        const { define, status } = context.getGlobalModuleRegistry();

        define((exports) => {
          exports({ foo: 'foo'})
        }, MODULE_IDS.foo, {}, true); // default behavior

        expect(status(MODULE_IDS.foo)).toBe('ready');
      });
    });

    describe('when the module has not been evaluated', () => {
      it('should returns `idle`', () => {
        const { define, status } = context.getGlobalModuleRegistry();

        define((exports) => {
          exports({ bar: 'bar' });
        }, MODULE_IDS.bar, {}, false);

        expect(status(MODULE_IDS.bar)).toBe('idle');
      });
    });

    describe('when the module was updated and has been evaluated', () => {
      it('should returns `ready`', () => {
        const { update, status } = context.getGlobalModuleRegistry();

        update(MODULE_IDS.foo, {}, true); // default behavior

        expect(status(MODULE_IDS.foo)).toBe('ready');
      });
    });

    describe('when the module was updated and has not been evaluated', () => {
      it('should returns `stale`', () => {
        const { update, status } = context.getGlobalModuleRegistry();

        update(MODULE_IDS.foo, {}, false);

        expect(status(MODULE_IDS.foo)).toBe('stale');
      });
    });
  });
});
