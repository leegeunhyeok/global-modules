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
      expect(typeof globalRegistry.register).toEqual('function');
      expect(typeof globalRegistry.getContext).toEqual('function');
      expect(typeof globalRegistry.clear).toEqual('function');
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

  describe('when register module', () => {
    let context: SandboxContext;

    beforeAll(() => {
      context = createSandboxContext();
      context.setup();
    });

    it('should return the module context', () => {
      expect(context.evaluate('__modules.register(0);')).toBeTruthy();
    });

    describe('when attempting to retrieve a registered module', () => {
      it('should return the module context', () => {
        expect(context.evaluate('__modules.getContext(0);')).toBeTruthy();
      });
    });

    describe('when attempting to retrieve a unregistered module', () => {
      it('should throw an error', () => {
        expect(
          (): RuntimeResult => context.evaluate('__modules.getContext(1234);'),
        ).toThrowError();
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
        var __ctx = __modules.register(1);
        __ctx.module.exports = 100;
      `);

      context.evaluate(`
        var __ctx = __modules.register(2);
        var mod = __ctx.require(1);
        print(mod);
      `);

      expect(mockedPrint).toBeCalledWith({
        default: 100,
      });
    });

    it('module.exports.named = <expr>;', () => {
      context.evaluate(`
        var __ctx = __modules.register(1);
        __ctx.module.exports.foo = 1;
        __ctx.module.exports.bar = 2;
        __ctx.module.exports.baz = 3;
      `);

      context.evaluate(`
        var __ctx = __modules.register(2);
        var mod = __ctx.require(1);
        print(mod);
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 1,
        bar: 2,
        baz: 3,
      });
    });

    it('Object.assign(module.exports, {});', () => {
      context.evaluate(`
        var __ctx = __modules.register(1);
        Object.assign(__ctx.module.exports, {
          foo: 4,
          bar: 5,
          baz: 6,
        });
      `);

      context.evaluate(`
        var __ctx = __modules.register(2);
        var mod = __ctx.require(1);
        print(mod);
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 4,
        bar: 5,
        baz: 6,
      });
    });

    it('Update the existing module', () => {
      context.evaluate(`
        var __ctx = __modules.register(1);
        __ctx.module.exports.foo = 1;
        __ctx.module.exports.bar = 2;
        __ctx.module.exports.baz = 3;
      `);

      context.evaluate(`
        var __ctx = __modules.register(2);
        var mod = __ctx.require(1);
        print(mod);
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 1,
        bar: 2,
        baz: 3,
      });

      // Update
      context.evaluate(`
        var __ctx = __modules.getContext(1);
        __ctx.module.exports.value = 100;
      `);

      context.evaluate(`
        var __ctx = __modules.getContext(2);
        var mod = __ctx.require(1);
        print(mod);
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
        var __ctx = __modules.register(1);
        var __x = 100;
        __ctx.exports(function () {
          return {
            default: __x,
          };
        });
        var __x;
      `);

      context.evaluate(`
        var __ctx = __modules.register(2);
        var mod = __ctx.require(1);
        print(mod);
      `);

      expect(mockedPrint).toBeCalledWith({
        default: 100,
      });
    });

    it('Named exports', () => {
      context.evaluate(`
        var __ctx = __modules.register(1);
        __x = 1;
        __x1 = 2;
        __x2 = 3;
        __ctx.exports(function () {
          return {
            foo: __x,
            bar: __x1,
            baz: __x2,
          };
        });
        var __x, __x1, __x2;
      `);

      context.evaluate(`
        var __ctx = __modules.register(2);
        var mod = __ctx.require(1);
        print(mod);
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 1,
        bar: 2,
        baz: 3,
      });
    });

    it('Re-export (named)', () => {
      context.evaluate(`
        var __ctx = __modules.register(1);
        __x = 1;
        __x1 = 2;
        __x2 = 3;
        __ctx.exports(function () {
          return {
            foo: __x,
            bar: __x1,
            baz: __x2,
          };
        });
        var __x, __x1, __x2;
      `);

      context.evaluate(`
        var __ctx = __modules.register(2);
        var mod = __ctx.require(1);
        __ctx.exports(function () {
          return {
            a: mod.foo,
            b: mod.bar,
            c: mod.baz,
          };
        });
      `);

      context.evaluate(`
        var __ctx = __modules.register(3);
        var mod = __ctx.require(2);
        print(mod);
      `);

      expect(mockedPrint).toBeCalledWith({
        a: 1,
        b: 2,
        c: 3,
      });
    });

    it('Re-export (all)', () => {
      context.evaluate(`
        var __ctx = __modules.register(1);
        __x = 1;
        __x1 = 2;
        __x2 = 3;
        __x3 = 4;
        __ctx.exports(function () {
          return {
            foo: __x,
            bar: __x1,
            baz: __x2,
            default: __x3,
          };
        });
        var __x, __x1, __x2, __x3;
      `);

      context.evaluate(`
        var __ctx = __modules.register(2);
        var mod = __ctx.require(1);
        print('require 1', mod);
        __ctx.exports(function () {
          return {
            ...__ctx.exports.ns(mod),
          };
        });
      `);

      context.evaluate(`
        var __ctx = __modules.register(3);
        var mod = __ctx.require(2);
        print('require 2', mod);
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
        var __ctx = __modules.register(1);
        __x = 1;
        __x1 = 2;
        __x2 = 3;
        __ctx.exports(function () {
          return {
            foo: __x,
            bar: __x1,
            baz: __x2,
          };
        });
        var __x, __x1, __x2;
      `);

      context.evaluate(`
        var __ctx = __modules.register(2);
        var mod = __ctx.require(1);
        print(mod);
      `);

      expect(mockedPrint).toBeCalledWith({
        foo: 1,
        bar: 2,
        baz: 3,
      });

      // Update
      context.evaluate(`
        var __ctx = __modules.getContext(1);
        __x = 100;
        __ctx.exports(function () {
          return {
            value: __x,
          };
        });
        var __x;
      `);

      context.evaluate(`
        var __ctx = __modules.getContext(2);
        var mod = __ctx.require(1);
        print(mod);
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
