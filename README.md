# global-modules

> [!NOTE]
> Global Modules provides a way to use modules in the global scope.

## Packages

- [@global-modules/runtime](https://github.com/leegeunhyeok/global-modules/tree/main/packages/runtime)
  - Register module's exports to the global module registry.
  - Reference the other module's exports.
- [@global-modules/swc-plugin](https://github.com/leegeunhyeok/global-modules/tree/main/packages/swc-plugin)
  - Transform plain module to the global module runtime specification.

## Usage

> For example, you can implement the HMR(Hot Module Replacement) via the global module.

First, setup the runtime environment.

```ts
import '@global-modules/runtime';

// ...
```

Then, transform the plain module to the global module runtime specification.

```ts
import * as swc from '@swc/core';
import plugin from '@global-modules/swc-plugin';

const result = await swc.transform(source, {
  jsc: {
    experimental: {
      plugins: [[plugin, { id: 'module-id', runtime: false }]],
    },
  },
});

result.code; // Transformed code is now following the global module runtime specification.
```

In runtime, you can reference the other module's exports by `global.__modules.require()`.

```js
const exports = global.__modules.require('module-id');

exports; // The module's exports.
```

For more details, see:

- [@global-modules/runtime](https://github.com/leegeunhyeok/global-modules/tree/main/packages/runtime)
- [@global-modules/swc-plugin](https://github.com/leegeunhyeok/global-modules/tree/main/packages/swc-plugin)

For more details about usage, see [demo implementation (HMR on esbuild)](https://github.com/leegeunhyeok/global-modules/tree/main/demo).

## License

[MIT](./LICENSE)
