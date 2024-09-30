import { describe, it, expect, beforeAll, vi, Mock } from 'vitest';
import { setup } from './index';

describe('@global-module/runtime', () => {
  const REGISTRY_NAME = '__modules';

  function getGlobalModuleRegistry(): GlobalModuleRegistry {
    return globalThis[REGISTRY_NAME];
  }

  describe('when call `setup()` once', () => {
    beforeAll(() => {
      delete globalThis[REGISTRY_NAME];
    });

    it('should define module registry into global context', () => {
      // Before setup
      expect(globalThis[REGISTRY_NAME]).toBeUndefined();

      setup({ registryName: REGISTRY_NAME });

      // After setup
      expect(globalThis[REGISTRY_NAME]).toBeTruthy();
      expect(typeof globalThis[REGISTRY_NAME].define).toEqual('function');
      expect(typeof globalThis[REGISTRY_NAME].update).toEqual('function');
    });

    describe('when call `setup()` twice', () => {
      it('should throw error', () => {
        expect(() => setup()).toThrowError();
      });
    });
  });

  describe('when define a module to global registry', () => {
    let mockedLog: Mock;

    beforeAll(() => {
      mockedLog = vi.fn();
      const { define } = getGlobalModuleRegistry();

      define(
        () => {
          mockedLog('hello, world');
        },
        0,
        {}
      );
    });

    it('should evaluate module immediately', () => {
      expect(mockedLog).toBeCalledTimes(1);
      expect(mockedLog).toBeCalledWith('hello, world');
    });
  });

  describe('when define modules to global registry', () => {
    // For simulate tests
    let exportsMap: Record<string, unknown> = {};
    let mockedEntryLog: Mock;

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
    beforeAll(() => {
      exportsMap = {};
      mockedEntryLog = vi.fn();

      const { define } = getGlobalModuleRegistry();

      // Module: d
      define((_$import, $exports) => {
        $exports.d = 10;
        exportsMap['./d'] = Object.assign({}, $exports);
      }, 5);

      // Module: c
      define(
        ($import, $exports) => {
          const mod = $import('./d');
          $exports.c = 20 + mod.d;
          exportsMap['./c'] = Object.assign({}, $exports);
        },
        4,
        {
          './d': () => exportsMap['./d'],
        }
      );

      // Module: b
      define(
        ($import, $exports) => {
          const mod = $import('./c');

          $exports.b = 30 + mod.c;
          exportsMap['./b'] = Object.assign({}, $exports);
        },
        3,
        {
          './c': () => exportsMap['./c'],
        }
      );

      // Module: a
      define((_$import, $exports) => {
        $exports.a = 100;
        exportsMap['./a'] = Object.assign({}, $exports);
      }, 2);

      // Entry
      define(
        ($import, _$exports) => {
          const mod0 = $import('./a');
          const mod1 = $import('./b');

          mockedEntryLog(mod0.a, mod1.b);
        },
        1,
        {
          './a': () => exportsMap['./a'],
          './b': () => exportsMap['./b'],
        }
      );
    });

    it('should evaluate module immediately', () => {
      expect(mockedEntryLog).toBeCalledTimes(1);
      expect(mockedEntryLog).toBeCalledWith(100, 60);
    });

    describe('when re-define specified module and update its reverse dependencies', () => {
      beforeAll(() => {
        console.log(exportsMap);
        Object.keys(exportsMap).forEach((key) => delete exportsMap[key]);
        console.log(exportsMap);

        const { define, update } = getGlobalModuleRegistry();

        // Module: d
        define((_$import, $exports) => {
          $exports.d = 100; // Change exports value
        }, 5);

        // Update reverse dependencies (parents)
        [
          { id: 4, ids: { './d': 5 } },
          { id: 3, ids: { './c': 4 } },
          { id: 2, ids: { './b': 3 } },
          { id: 1, ids: { './a': 2 } },
        ].forEach(({ id, ids }) => {
          update(id, ids);
        });
      });

      it('should re-evaluate each modules', () => {
        // 2nd call
        expect(mockedEntryLog).toBeCalledTimes(2);
        expect(mockedEntryLog).toBeCalledWith(100, 150);
      });
    });
  });
});
