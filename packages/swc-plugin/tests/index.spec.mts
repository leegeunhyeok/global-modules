import assert from 'node:assert';
import * as path from 'node:path';
import * as vm from 'node:vm';
import * as swc from '@swc/core';
import * as esbuild from 'esbuild';
import { vi, describe, it, expect } from 'vitest';
import { Phase } from '../types.js';

async function bundle(
  entryCode: string,
  code: string,
  { phase }: { phase: Phase },
) {
  const result = await swc.transformSync(code, {
    jsc: {
      target: 'es5',
      experimental: {
        plugins: [
          [
            path.resolve(
              import.meta.dirname,
              '../swc_plugin_global_modules.wasm',
            ),
            { id: '0', phase },
          ],
        ],
      },
    },
  });

  const buildResult = await esbuild.build({
    absWorkingDir: '/',
    bundle: true,
    write: false,
    format: 'iife',
    stdin: {
      contents: entryCode,
      loader: 'js',
    },
    plugins: [
      {
        name: 'linker',
        setup(build) {
          build.onResolve({ filter: /.*/ }, async () => ({
            path: '@',
            namespace: 'linker',
          }));

          build.onLoad({ filter: /.*/, namespace: 'linker' }, async () => ({
            contents: result.code,
            loader: 'js',
          }));
        },
      },
    ],
  });

  const bundleCode = buildResult.outputFiles[0]?.text;
  assert(bundleCode, 'invalid bundle result');

  return bundleCode;
}

function evaluate(code: string, context?: vm.Context) {
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

  const runtimeContext = vm.createContext({
    global: { __modules: { register, getContext: register } },
    ...context,
  });

  return new vm.Script(code).runInContext(runtimeContext);
}

describe('@global-modules/swc-plugin', () => {
  describe('Bundle phase', () => {
    it('[ESM] Basics', async () => {
      const bundleCode = await bundle(
        `
        import * as mod from '.';
      
        bridge(mod);
        `,
        `
        const foo = 'foo';
        const bar = 'bar';
        const value = 'baz';

        export default 1;
        export const foo = 'foo';
        export { bar, value as baz };
        `,
        { phase: Phase.Bundle },
      );

      const bridge = vi.fn();
      evaluate(bundleCode, { bridge });

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
        `
        import * as mod from '.';

        bridge(mod.newObj);
        `,
        `
        const obj = { value: 0 };
        export const newObj = obj;

        newObj.key = 'key';
        `,
        { phase: Phase.Bundle },
      );

      const bridge = vi.fn();
      evaluate(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ value: 0, key: 'key' });
    });

    it('[CJS] Basics', async () => {
      const bundleCode = await bundle(
        `
        const mod = require('.');

        bridge(mod);
        `,
        `
        const foo = 'foo';
        const bar = 'bar';
        const value = 'baz';

        if (true) {
          module.exports = { foo, bar, baz: value };
        }
        `,
        { phase: Phase.Bundle },
      );

      const bridge = vi.fn();
      evaluate(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ foo: 'foo', bar: 'bar', baz: 'baz' });
    });
  });

  describe('Runtime phase', () => {
    it('[ESM] Basics', async () => {
      const bundleCode = await bundle(
        `
        import * as mod from '.';

        bridge(global.__modules.getContext(0).module.exports);
        `,
        `
        const foo = 'foo';
        const bar = 'bar';
        const value = 'baz';

        export default 1;
        export const foo = 'foo';
        export { bar, value as baz };
        `,
        { phase: Phase.Runtime },
      );

      const bridge = vi.fn();
      evaluate(bundleCode, { bridge });

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
        `
        import * as mod from '.';

        bridge(global.__modules.getContext(0).module.exports);
        `,
        `
        const obj = { value: 0 };
        export const newObj = obj;

        newObj.key = 'key';
        `,
        { phase: Phase.Runtime },
      );

      const bridge = vi.fn();
      evaluate(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ newObj: { value: 0, key: 'key' } });
    });

    it('[CJS] Basics', async () => {
      const bundleCode = await bundle(
        `
        const mod = require('.');

        bridge(global.__modules.getContext(0).module.exports);
        `,
        `
        const foo = 'foo';
        const bar = 'bar';
        const value = 'baz';

        if (true) {
          module.exports = { foo, bar, baz: value };
        }
        `,
        { phase: Phase.Runtime },
      );

      const bridge = vi.fn();
      evaluate(bundleCode, { bridge });

      expect(bundleCode).toMatchSnapshot();
      expect(bridge).toBeCalledWith({ foo: 'foo', bar: 'bar', baz: 'baz' });
    });
  });
});
