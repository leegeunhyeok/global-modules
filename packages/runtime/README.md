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

export function baz() {
  return foo.value + bar.value;
}
```

```ts
// Transform module body to global module.
import foo from './foo';
import bar from './bar';

global.__modules.define(
  (exports, require) => {
    const mod0 = require(0); // Index of dependencies (foo)
    const mod1 = require(1); // Index of dependencies (bar)

    function baz() {
      return mod0.value + bar.value;
    }

    exports.baz = baz;
  },
  0, // Module ID
  [foo, bar], // Dependencies
);

// Re-evaluate target module with its dependencies IDs. (For example: when an HMR update is accepted, etc.)
global.__modules.update(0, [1, 2]);
```

For transform to global module, see more: [@global-modules/swc-plugin](https://github.com/leegeunhyeok/global-modules/tree/main/packages/swc-plugin)
