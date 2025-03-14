import vm from 'node:vm';
import fs from 'node:fs/promises';
import path from 'node:path';
import { describe, it, expect, beforeAll, beforeEach, vi } from 'vitest';
import type { GlobalModule } from './types.js';

type RuntimeResult = unknown;

const runtimeCode = await fs.readFile(
  path.resolve(import.meta.dirname, '../dist/index.js'),
  {
    encoding: 'utf-8',
  },
);

describe('@global-modules/runtime', () => {
  function createSandboxContext(sandbox?: vm.Context): SandboxContext {
    const context = vm.createContext({
      ...sandbox,
    });

    function evaluate(code: string): RuntimeResult {
      return new vm.Script(code).runInContext(context);
    }

    return {
      setup: (): RuntimeResult => {
        return evaluate(runtimeCode);
      },
      getGlobalModule: (): GlobalModule => {
        return evaluate(
          '(new Function("return this")())["__modules"];',
        ) as GlobalModule;
      },
      evaluate,
    };
  }

  interface SandboxContext {
    setup: () => void;
    getGlobalModule: () => GlobalModule;
    evaluate: (code: string) => RuntimeResult;
  }

  describe('when import runtime module', () => {
    let context: SandboxContext;

    beforeAll(() => {
      context = createSandboxContext();
      context.setup();
    });

    it('should define module registry into global context', () => {
      const globalRegistry = context.getGlobalModule();
      expect(typeof globalRegistry.define).toEqual('function');
      expect(typeof globalRegistry.apply).toEqual('function');
      expect(typeof globalRegistry.require).toEqual('function');
      expect(typeof globalRegistry.clear).toEqual('function');
      expect(typeof globalRegistry.getRegistry).toEqual('function');
    });
  });

  describe('when `__modules` property is already defined in the global context', () => {
    let context: SandboxContext;

    beforeAll(() => {
      context = createSandboxContext({ __modules: '' });
    });

    it('should throw an error', () => {
      expect(() => context.setup()).toThrowError();
    });
  });

  describe('when define the module', () => {
    let context: SandboxContext;

    beforeAll(() => {
      context = createSandboxContext();
      context.setup();
    });

    it('should return the module context', () => {
      context.evaluate(`__modules.define(() => {}, 'mod-0');`);
      expect(context.evaluate(`__modules.require('mod-0');`)).toBeTruthy();
    });

    describe('when attempting to retrieve a unregistered module', () => {
      it('should throw an error', () => {
        expect(
          (): RuntimeResult =>
            context.evaluate(`__modules.require('mod-unregistered');`),
        ).toThrowError();
      });
    });
  });

  describe('when define the module with dependency', () => {
    const mockedPrint = vi.fn();
    let context: SandboxContext;

    beforeAll(() => {
      mockedPrint.mockReset();
      context = createSandboxContext({ print: mockedPrint });
      context.setup();
    });

    it('should reference the provided dependency', () => {
      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('react');
          print(mod);
        }, 'mod-0', {
          'react': () => ({
            default: 'React',
            useState: 'hook#useState',
            useEffect: 'hook#useEffect',
          }),
        });
      `);

      expect(mockedPrint).toBeCalledWith({
        default: 'React',
        useState: 'hook#useState',
        useEffect: 'hook#useEffect',
      });
    });
  });

  describe('when apply the module with dependency id map', () => {
    const mockedPrint = vi.fn();
    let context: SandboxContext;

    beforeAll(() => {
      mockedPrint.mockReset();
      context = createSandboxContext({ print: mockedPrint });
      context.setup();
    });

    it('should reference the module that provided id', () => {
      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('./foo');
          print('require foo', mod);
        }, 'mod', {
          './foo': () => ({ default: 'foo' }),
        });
      `);

      expect(mockedPrint).toBeCalledWith('require foo', { default: 'foo' });

      context.evaluate(`
        __modules.define((context) => {
          context.exports(() => ({ default: 'defined foo', foo: 'foo' }));
        }, 'foo-id');
        __modules.apply('mod', { './foo': 'foo-id' });
      `);

      // `context.require('./foo')` should reference the re-mapped module (`foo-id`).
      expect(mockedPrint).toBeCalledWith('require foo', {
        default: 'defined foo',
        foo: 'foo',
      });
    });
  });

  describe('CommonJS', () => {
    const mockedPrint = vi.fn();
    let context: SandboxContext;

    beforeEach(() => {
      mockedPrint.mockReset();
      context = createSandboxContext({ print: mockedPrint });
      context.setup();
    });

    it('module.exports = <expr>;', () => {
      context.evaluate(`
        __modules.define((context) => {
          context.module.exports = 100;
        }, 'cjs-1');
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('cjs-1');
          print(mod);
        }, 'cjs-2');
      `);

      expect(mockedPrint).toBeCalledWith({ default: 100 });
    });

    it('module.exports.named = <expr>;', () => {
      context.evaluate(`
        __modules.define((context) => {
          context.module.exports.foo = 1;
          context.module.exports.bar = 2;
          context.module.exports.baz = 3;
        }, 'cjs-1');
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('cjs-1');
          print(mod);
        }, 'cjs-2');
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 1,
        bar: 2,
        baz: 3,
      });
    });

    it('Object.assign(module.exports, {});', () => {
      context.evaluate(`
        __modules.define((context) => {
          Object.assign(context.module.exports, {
            foo: 4,
            bar: 5,
            baz: 6,
          });
        }, 'cjs-1');
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('cjs-1');
          print(mod);
        }, 'cjs-2');
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 4,
        bar: 5,
        baz: 6,
      });
    });

    it('Update the existing module', () => {
      context.evaluate(`
        __modules.define((context) => {
          context.module.exports.foo = 1;
          context.module.exports.bar = 2;
          context.module.exports.baz = 3;
        }, 'cjs-1');
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('cjs-1');
          print(mod);
        }, 'cjs-2');
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 1,
        bar: 2,
        baz: 3,
      });

      // Update
      context.evaluate(`
        __modules.define((context) => {
          context.module.exports.value = 100;
        }, 'cjs-1');
        __modules.apply('cjs-2');
      `);

      expect(mockedPrint).toBeCalledWith({
        // Before:
        // foo: 1,
        // bar: 2,
        // baz: 3,
        //
        // After:
        value: 100,
      });
    });

    it('ESModule interop', () => {
      context.evaluate(`
        __modules.define((context) => {
          context.module.exports = {
            default: 100,
          };
          Object.defineProperty(context.module.exports, '__esModule', {
            value: true,
          });
        }, 'cjs-1');
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('cjs-1');
          print(mod);
        }, 'cjs-2');
      `);

      expect(mockedPrint).toBeCalledWith({
        default: 100,
      });
    });
  });

  describe('ESModule', () => {
    const mockedPrint = vi.fn();
    let context: SandboxContext;

    beforeEach(() => {
      mockedPrint.mockReset();
      context = createSandboxContext({ print: mockedPrint });
      context.setup();
    });

    it('Default export', () => {
      context.evaluate(`
        __modules.define((context) => {
          __x = 100;
          context.exports(() => ({
            default: __x,
          }));
        }, 'esm-1');
        var __x;
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('esm-1');
          print(mod);
        }, 'esm-2');
      `);

      expect(mockedPrint).toBeCalledWith({
        default: 100,
      });
    });

    it('Named exports', () => {
      context.evaluate(`
        __modules.define((context) => {
          __x = 1;
          __x1 = 2;
          __x2 = 3;
          context.exports(() => ({
            foo: __x,
            bar: __x1,
            baz: __x2,
          }));
        }, 'esm-1');
        var __x, __x1, __x2;
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('esm-1');
          print(mod);
        }, 'esm-2');
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 1,
        bar: 2,
        baz: 3,
      });
    });

    it('Re-export (named)', () => {
      context.evaluate(`
        __modules.define((context) => {
          __x = 1;
          __x1 = 2;
          __x2 = 3;
          context.exports(() => ({
            foo: __x,
            bar: __x1,
            baz: __x2,
          }));
        }, 'esm-1');
        var __x, __x1, __x2;
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('esm-1');
          context.exports(() => ({
            a: mod.foo,
            b: mod.bar,
            c: mod.baz,
          }));
        }, 'esm-2');
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('esm-2');
          print(mod);
        }, 'esm-3');
      `);

      expect(mockedPrint).toBeCalledWith({
        a: 1,
        b: 2,
        c: 3,
      });
    });

    it('Re-export (all)', () => {
      context.evaluate(`
        __modules.define((context) => {
          __x = 1;
          __x1 = 2;
          __x2 = 3;
          __x3 = 4;
          context.exports(() => ({
            foo: __x,
            bar: __x1,
            baz: __x2,
            default: __x3,
          }));
        }, 'esm-1');
        var __x, __x1, __x2, __x3;
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('esm-1');
          print('require 1', mod);
          context.exports(() => ({
            ...context.exports.ns(mod),
          }));
        }, 'esm-2');
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('esm-2');
          print('require 2', mod);
        }, 'esm-3');
      `);

      expect(mockedPrint).toBeCalledWith('require 1', {
        foo: 1,
        bar: 2,
        baz: 3,
        default: 4,
      });
      expect(mockedPrint).toBeCalledWith('require 2', {
        foo: 1,
        bar: 2,
        baz: 3,
        // The `default` field should be excluded.
      });
    });

    it('Update the existing module', () => {
      context.evaluate(`
        __modules.define((context) => {
          __x = 1;
          __x1 = 2;
          __x2 = 3;
          context.exports(() => ({
            foo: __x,
            bar: __x1,
            baz: __x2,
          }));
        }, 'esm-1');
        var __x, __x1, __x2;
      `);

      context.evaluate(`
        __modules.define((context) => {
          const mod = context.require('esm-1');
          print(mod);
        }, 'esm-2');
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 1,
        bar: 2,
        baz: 3,
      });

      // Update
      context.evaluate(`
        __modules.define((context) => {
          __x = 100;
          context.exports(() => ({
            value: __x,
          }));
        }, 'esm-1');
        var __x;
        __modules.apply('esm-2');
      `);

      expect(mockedPrint).toBeCalledWith({
        // Before:
        // foo: 1,
        // bar: 2,
        // baz: 3,
        //
        // After:
        value: 100,
      });
    });
  });
});
