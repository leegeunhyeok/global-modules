# swc-plugin

A [SWC](https://swc.rs) plugin that transforms code to comply with the [@global-modules/runtime](https://github.com/leegeunhyeok/global-modules/tree/main/packages/runtime) specifications.

## Usage

```ts
import * as swc from '@swc/core';
import plugin from '@global-modules/swc-plugin';

await swc.transform(code, {
  jsc: {
    experimental: {
      plugins: [
        [
          plugin,
          {
            // The module's id.
            id: 'module-id',
            // Transform as runtime module.
            //
            // - `false`: Bundle phase.
            // - `true`: Runtime phase.
            runtime: false,
            // The paths for mapping module sources.
            paths: {
              react: 'react-module-id',
              './Container': 'container-module-id',
            },
          },
        ],
      ],
    },
  },
});
```

### Options

| Option    | Type                     | Description                               | Required |
| --------- | ------------------------ | ----------------------------------------- | -------- |
| `id`      | `string`                 | The module's unique identifier.           | O        |
| `runtime` | `boolean`                | The flag for transform as runtime module. | O        |
| `paths`   | `Record<string, string>` | The paths for mapping module sources.     |          |

- `runtime: false`: Register only the module's exports. At this phase, the module statements(ESM: `import`, `export` / CommonJS: `require`, `module`) are not transformed, as these are delegated to the bundler to follow its module resolution specification.
- `runtime: true`: Register the module's exports and strip module statements. At this phase, module reference statements are transformed into the global module's require call expression(`global.__modules.require()`) to reference other modules' exports at runtime.

|                         | Bundle Phase | Runtime Phase |
| ----------------------- | ------------ | ------------- |
| Register exports        | ✅           | ✅            |
| Strip module statements | ❌           | ✅            |

## Preview

```ts
// Original source
import React, { useState, useCallback } from 'react';
import { Component } from './Container';

export function Component() {
  // ...
}
```

<details>

<summary>Bundle phase</summary>

```ts
import React, { useState, useCallback } from 'react';
import { Component } from './Container';
r __ctx = global.__modules.register('1');
function Component() {
  // ...
}
__x = Component;
__ctx.exports(function () {
  return {
    Component: __x,
  };
});
var __x;
export { __x as Component };
```

</details>

<details>

<summary>Runtime phase</summary>

````ts
/**
 * With `paths`
 *
 * ```js
 * {
 *   "react": "react-module-id",
 *   "./Container": "container-module-id",
 * }
 * ```
 */
var __ctx = global.__modules.register('1');
var {
  default: React,
  useState,
  useCallback,
} = global.__modules.require('react-module-id');
var { Component } = global.__modules.require('container-module-id');
function Component() {
  // ...
}
__x = Component;
__ctx.exports(function () {
  return {
    Component: __x,
  };
});
var __x;
````

</details>

## License

[MIT](./LICENSE)
