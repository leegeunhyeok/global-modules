# swc-plugin

A SWC plugin that transforms according to the [@global-modules/runtime](https://github.com/leegeunhyeok/global-modules/tree/main/packages/runtime) specifications.

## Usage

```ts
import * as swc from '@swc/core';
import globalModulePlugin, { Phase } from '@global-modules/swc-plugin';

await swc.transform(code, {
  jsc: {
    experimental: {
      plugins: [
        [
          globalModulePlugin,
          {
            id: 1,
            phase: Phase.Register, // or Phase.Runtime
            // ID values used to replace the original sources
            dependencyIds: {
              react: 1000,
              './Container': 1234,
            },
          },
        ],
      ],
    },
  },
});
```

```ts
// Original source
import React, { useState, useCallback } from 'react';
import { Component } from './Container';

export function Component() {
  // ...
}
```

<details>

<summary>Phase.Register</summary>

```ts
var __ctx = global.__modules.register(1);
import React, { useState, useCallback } from 'react';
import { Component } from './Container';
export { __x as Component };
__x = function Component() {
  // ...
};
__ctx.exports(function () {
  return {
    Component: __x,
  };
});
var __x;
```

</details>

<details>

<summary>Phase.Runtime</summary>

```ts
var __ctx = global.__modules.getContext(1);
var { default: React, useState, useCallback } = __ctx.require(1000);
var { Component } = __ctx.require('./Container');
__x = function Component() {
  // ...
};
__ctx.exports(function () {
  return {
    Component: __x,
  };
});
var __x;
```

</details>
