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
            // `Phase.Bundle` or `Phase.Runtime`.
            phase: Phase.Bundle,
            // ID values used to replace the original sources.
            paths: {
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

<summary>Phase.Bundle</summary>

```ts
import React, { useState, useCallback } from 'react';
import { Component } from './Container';
var __ctx = global.__modules.register(1);
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
var __ctx = global.__modules.getContext(1);
__ctx.reset();
var { default: React, useState, useCallback } = global.__modules.require(1000);
var { Component } = global.__modules.require(1234);
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
