# runtime

Configure the runtime environment to register the module in the global module repository.

## Usage

Setup from top of entry point like this:

```ts
import { setup } from '@global-modules/runtime';

setup();

global.__modules.define(
  () => {
    // Module body
  },
  0, // Module ID
  [deps0, deps1, deps2], // Module's dependencies
);

global.__modules.update(0, [deps0, deps1, deps2]);
```

- `define`: Define the module and register it in the global registry. It can be re-evaluated later using global module APIs.
- `update`: Re-evaluate the specified module registered in the global registry.

## Concept

```ts
// Original source
import foo from './foo';
import bar from './bar';
import { baz } from './baz';

export function something() {
  return foo.value + bar.value + baz;
}
```

```ts
// Transform module body to global module.
import foo from './foo';
import bar from './bar';
import { baz } from './baz';

_i0 = { default: foo };
_i1 = { default: bar };
_i2 = { baz: baz };

global.__modules.define(
  (exports, require) => {
    const mod0 = require(0); // foo
    const mod1 = require(1); // bar
    const mod2 = require(2); // baz

    exports.something = _x0 = function something() {
      return mod0.default.value + mod1.default.value + mod2.baz;
    };
  },
  0, // Module ID
  [_i0, _i1, _i2], // Dependencies
);

var _i0, _i1, i2;
var _x0;

export { _x0 as baz };
```

```ts
// Re-evaluate target module with its dependencies IDs. (For example: when an HMR update is accepted, etc.)
global.__modules.update(
  0,
  [
    1 /* foo's ID */,
    2 /* bar's ID */,
    3 /* baz's ID */,
  ],
);
```

For transform to global module, see more: [@global-modules/swc-plugin](https://github.com/leegeunhyeok/global-modules/tree/main/packages/swc-plugin)
