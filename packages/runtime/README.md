# runtime

Configure the runtime environment to register and import modules in the global module registry.

## Usage

Setup from top of entry point like this:

```ts
import '@global-modules/runtime';

// ...
```

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

Register the module's exports to the global module registry using `context.exports()`.

```ts
// 1. Bundle phase
import foo from './foo';
import bar from './bar';
import { baz } from './baz';

global.__modules.define(function (__context) {
  const { default: foo } = __context.require('./foo', 0);
  const { default: bar } = __context.require('./bar', 1);
  const { baz } = __context.require('./baz', 2);

  function something() {
    return foo.value + bar.value + baz;
  }

  __x = something;
  __ctx.exports(function () {
    return {
      something: __x,
    };
  });
});
var __x;

export { __x as something };
```

Reference other module's exports using `context.require()` or `global.__modules.require()`.

```ts
// 2. Runtime phase
global.__modules.define(function (__context) {
  const { default: foo } = __context.require('1000', 0); // `./foo` module's id
  const { default: bar } = __context.require('1001', 1); // `./bar` module's id
  const { baz } = __context.require('1002', 2); // `./baz` module's id

  function something() {
    return foo.value + bar.value + baz;
  }
  __x = something;
  __ctx.exports(function () {
    return {
      something: __x,
    };
  });
});
var __x;
```

For transform plain module to the global module runtime specification, see more: [@global-modules/swc-plugin](https://github.com/leegeunhyeok/global-modules/tree/main/packages/swc-plugin)

## Specification

See [SPECIFICATION.md](./SPECIFICATION.md)

### define

The `define()` method registers a module context.

```ts
// Signature
type Define = (
  factory: (context: ModuleContext) => void,
  id: ModuleId,
  dependencies?: (() => unknown)[] | null,
) => void;

// Example
global.__modules.define(
  function (context) {
    // Actual module body
  },
  'module-id',
  [() => dep1, () => dep2],
);
```

### require

The `require()` method returns the exports object of the module. if the module it not registered, it will throw an error.

```ts
// Signature
type Require = (id: ModuleId) => Exports;

// Example
const exports = global.__modules.require('module-id');
```

### apply

The `apply()` method re-evaluates the module factory.
If the module's dependencies are updated, you can use this method to re-evaulate the module.

If dependency id map is provided, the module will be re-evaluated with the replaced module sources.

```ts
// Signature
type Apply = (id: ModuleId, dependencyMap?: Record<string, string>) => void;

// Example
global.__modules.apply('module-id', { react: 'react-id', './mod': 'mod-id' });

// In the module factory
context.require('react', 0); // Reference 'react-id' instead of 'react'
context.require('./mod', 1); // Reference 'mod-id' instead of './mod'
```

### clear

The `clear()` method clears all modules from the global module registry.

```ts
// Signature
type Clear = () => void;

// Example
global.__modules.clear();
```

### getModule

The `getModule()` method returns the module from the global module registry.
You can use this method to store the some metadata to the module.

```ts
// Signature
type GetModule = (id: ModuleId) => Module;

// Example
const module = global.__modules.getModule('module-id');

module.meta = { value: 'foo' };
```

### getRegistry

The `getRegistry()` method returns the global module registry.

```ts
// Signature
type GetRegistry = () => Map<ModuleId, Module>;

// Example
const registry = global.__modules.getRegistry();
```

## License

[MIT](./LICENSE)
