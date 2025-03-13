# swc-plugin

A [SWC](https://swc.rs) plugin that transforms code to comply with the [@global-modules/runtime](https://github.com/leegeunhyeok/global-modules/tree/main/packages/runtime) specifications.

## Usage

```ts
import * as swc from '@swc/core';
import plugin, { Phase } from '@global-modules/swc-plugin';

await swc.transform(code, {
  jsc: {
    experimental: {
      plugins: [
        [
          plugin,
          {
            // The module's id.
            id: 'module-id',
            // `Phase.Bundle` or `Phase.Runtime`.
            phase: Phase.Bundle,
          },
        ],
      ],
    },
  },
});
```

### Options

| Option  | Type                     | Description                                | Required |
| ------- | ------------------------ | ------------------------------------------ | -------- |
| `id`    | `string`                 | The module's unique identifier.            | O        |
| `phase` | `Phase`                  | The phase of the plugin.                   | O        |

- `Phase.Bundle`: Register only the module's exports. At this phase, the module statements(ESM: `import`, `export` / CommonJS: `require`, `module`) are not transformed, as these are delegated to the bundler to follow its module resolution specification.
- `Phase.Runtime`: Register the module's exports and strip module statements. At this phase, module reference statements are transformed into the global module's require call expression(`global.__modules.require()`) to reference other modules' exports at runtime.

|                         | Phase.Bundle | Phase.Runtime |
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

<summary>Phase.Bundle</summary>

```ts
import React, { useState, useCallback } from 'react';
import { Component } from './Container';
var __ctx = global.__modules.register('1');
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

<summary>Phase.Runtime</summary>

```ts
var __ctx = global.__modules.getContext('1');
__ctx.reset();
var {
  default: React,
  useState,
  useCallback,
} = global.__modules.require('1000');
var { Component } = global.__modules.require('1234');
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
```

</details>
