import * as vm from 'node:vm';
import { vi, describe, it, expect } from 'vitest';
import { Phase } from '../types.js';
import { bundle } from './utils/bundle.js';
import { evaluateOnSandbox } from './utils/sandbox.js';

function evaluateOnGlobalModuleSandbox(code: string, context?: vm.Context) {
  const modules = {};

  function register(id) {
    if (modules[id]) {
      return { ...modules[id], reset: vi.fn() };
    }

    const module = { exports: {} };

    modules[id] = { module };

    return {
      module,
      exports: (factory) => {
        modules[id].module.exports = factory();
      },
      reset: vi.fn(),
    };
  }

  return evaluateOnSandbox(code, {
    global: { __modules: { register, getContext: register } },
    ...context,
  });
}

describe('@global-modules/swc-plugin', () => {
  describe('Bundle phase', () => {
    it('[ESM] Basics', async () => {
      const bundleCode = await bundle(
        {
          entry: `
          import * as mod from './foo';
        
          bridge(mod);
          `,
          foo: `
          const foo = 'foo';
          const bar = 'bar';
          const value = 'baz';

          export default 1;
          export const foo = 'foo';
          export { bar, value as baz };
          `,
        },
        { phase: Phase.Bundle },
      );

      const bridge = vi.fn();
      evaluateOnGlobalModuleSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({
        default: 1,
        foo: 'foo',
        bar: 'bar',
        baz: 'baz',
      });
    });

    it('[ESM] Export with declaration statements', async () => {
      const bundleCode = await bundle(
        {
          entry: `
          import * as mod from './foo';

          bridge(mod.newObj);
          `,
          foo: `
          const obj = { value: 0 };
          export const newObj = obj;

          newObj.key = 'key';
          `,
        },
        { phase: Phase.Bundle },
      );

      const bridge = vi.fn();
      evaluateOnGlobalModuleSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ value: 0, key: 'key' });
    });

    it('[CJS] Basics', async () => {
      const bundleCode = await bundle(
        {
          entry: `
          const mod = require('./foo');

          bridge(mod);
          `,
          foo: `
          const foo = 'foo';
          const bar = 'bar';
          const value = 'baz';

          if (true) {
            module.exports = { foo, bar, baz: value };
          }
          `,
        },
        { phase: Phase.Bundle },
      );

      const bridge = vi.fn();
      evaluateOnGlobalModuleSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ foo: 'foo', bar: 'bar', baz: 'baz' });
    });
  });

  describe('Runtime phase', () => {
    it('[ESM] Basics', async () => {
      const bundleCode = await bundle(
        {
          entry: `
          import * as mod from './foo';

          bridge(global.__modules.getContext(0).module.exports);
          `,
          foo: `
          const foo = 'foo';
          const bar = 'bar';
          const value = 'baz';

          export default 1;
          export const foo = 'foo';
          export { bar, value as baz };
          `,
        },
        { phase: Phase.Runtime },
      );

      const bridge = vi.fn();
      evaluateOnGlobalModuleSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({
        default: 1,
        foo: 'foo',
        bar: 'bar',
        baz: 'baz',
      });
    });

    it('[ESM] Export with declaration statements', async () => {
      const bundleCode = await bundle(
        {
          entry: `
          import * as mod from './foo';

          bridge(global.__modules.getContext(0).module.exports);
          `,
          foo: `
          const obj = { value: 0 };
          export const newObj = obj;

          newObj.key = 'key';
          `,
        },
        { phase: Phase.Runtime },
      );

      const bridge = vi.fn();
      evaluateOnGlobalModuleSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ newObj: { value: 0, key: 'key' } });
    });

    it('[CJS] Basics', async () => {
      const bundleCode = await bundle(
        {
          entry: `
          const mod = require('./foo');

          bridge(global.__modules.getContext(0).module.exports);
          `,
          foo: `
          const foo = 'foo';
          const bar = 'bar';
          const value = 'baz';

          if (true) {
            module.exports = { foo, bar, baz: value };
          }
          `,
        },
        { phase: Phase.Runtime },
      );

      const bridge = vi.fn();
      evaluateOnGlobalModuleSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ foo: 'foo', bar: 'bar', baz: 'baz' });
    });
  });
});
