import { vi, describe, it, expect } from 'vitest';
import { bundle, bundleWithFoo } from './utils/bundle.js';
import { evaluateOnSandbox } from './utils/sandbox.js';
import { tests } from './module-tests.mjs';

describe('@global-modules/swc-plugin', () => {
  describe('Bundle phase', () => {
    it('[ESM] Basics', async () => {
      const bundleCode = await bundleWithFoo(
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
        { runtime: false },
      );

      const bridge = vi.fn();
      evaluateOnSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({
        default: 1,
        foo: 'foo',
        bar: 'bar',
        baz: 'baz',
      });
    });

    it('[ESM] Export with declaration statements', async () => {
      const bundleCode = await bundleWithFoo(
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
        { runtime: false },
      );

      const bridge = vi.fn();
      evaluateOnSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ value: 0, key: 'key' });
    });

    it('[CJS] Basics', async () => {
      const bundleCode = await bundleWithFoo(
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

          module.exports[(() => 'qux')()] = 'qux';

          // These are invalid commonjs module expressions.
          function invalidCommonJS(module) {
            var require = () => {};

            module.exports.invalid = 'invalid';
            module.exports = 'default';

            require('./some-module');
          }

          invalidCommonJS({ exports: {} });
          `,
        },
        { runtime: false },
      );

      const bridge = vi.fn();
      evaluateOnSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({
        foo: 'foo',
        bar: 'bar',
        baz: 'baz',
        qux: 'qux',
      });
      expect(bridge).not.toBeCalledWith({ invalid: 'invalid' });
    });
  });

  describe('Runtime phase', () => {
    it('[ESM] Basics', async () => {
      const bundleCode = await bundleWithFoo(
        {
          entry: `
          import * as mod from './foo';

          bridge(global.__modules.getContext('0').module.exports);
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
        { runtime: true },
      );

      const bridge = vi.fn();
      evaluateOnSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({
        default: 1,
        foo: 'foo',
        bar: 'bar',
        baz: 'baz',
      });
    });

    it('[ESM] Export with declaration statements', async () => {
      const bundleCode = await bundleWithFoo(
        {
          entry: `
          import * as mod from './foo';

          bridge(global.__modules.getContext('0').module.exports);
          `,
          foo: `
          const obj = { value: 0 };
          export const newObj = obj;

          newObj.key = 'key';
          `,
        },
        { runtime: true },
      );

      const bridge = vi.fn();
      evaluateOnSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ newObj: { value: 0, key: 'key' } });
    });

    it('[CJS] Basics', async () => {
      const bundleCode = await bundleWithFoo(
        {
          entry: `
          const mod = require('./foo');

          bridge(global.__modules.getContext('0').module.exports);
          `,
          foo: `
          const foo = 'foo';
          const bar = 'bar';
          const value = 'baz';

          if (true) {
            module.exports = { foo, bar, baz: value };
          }

          module.exports[(() => 'qux')()] = 'qux';

          // These are invalid commonjs module expressions.
          function invalidCommonJS(module) {
            var require = () => {};

            module.exports.invalid = 'invalid';
            module.exports = 'default';

            require('./some-module');
          }

          invalidCommonJS({ exports: {} });
          `,
        },
        { runtime: true },
      );

      const bridge = vi.fn();
      evaluateOnSandbox(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({
        foo: 'foo',
        bar: 'bar',
        baz: 'baz',
        qux: 'qux',
      });
      expect(bridge).not.toBeCalledWith({ invalid: 'invalid' });
    });
  });

  describe.each(tests)('Module tests', (input) => {
    const index = tests.indexOf(input);

    it(`Case #${index}`, async () => {
      const bundleCode = await bundle(input, { index, runtime: false });

      const resultBridgeObject = {
        input: {
          works: undefined,
        },
      };

      evaluateOnSandbox(bundleCode, resultBridgeObject);

      expect(await Promise.resolve(resultBridgeObject.input.works)).toBe(true);
    });
  });
});
