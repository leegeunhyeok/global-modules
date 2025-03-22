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

const __ctx = global.__modules.context('1');

function something() {
  return foo.value + bar.value + baz;
}

__x = something;
__ctx.exports(function () {
  return {
    something: __x,
  };
});
var __x;

export { __x as something };
```

Reference other module's exports using `global.__modules.require()`.

```ts
// 2. Runtime phase
var __ctx = global.__modules.context('1');

var { default: foo } = global.__modules.require('1000'); // `./foo` module's id
var { default: bar } = global.__modules.require('1001'); // `./bar` module's id
var { baz } = global.__modules.require('1002'); // `./baz` module's id

function something() {
  return foo.value + bar.value + baz;
}
__x = something;
__ctx.exports(function () {
  return {
    something: __x,
  };
});
var __x;
```

For transform plain module to the global module runtime specification, see more: [@global-modules/swc-plugin](https://github.com/leegeunhyeok/global-modules/tree/main/packages/swc-plugin)

## Specification

See [SPECIFICATION.md](./SPECIFICATION.md)

### context

The `context()` method registers the module context or returns the existing module context.

```ts
// Signature
type Context = (id: ModuleId) => ModuleContext;

// Example
const __ctx = global.__modules.context('1');
```

### require

The `require()` method returns the exports object of the module. if the module it not registered, it will throw an error.

```ts
// Signature
type Require = (id: ModuleId) => Exports;

// Example
const exports = global.__modules.require('module-id');
```

### clear

The `clear()` method clears all modules from the global module registry.

```ts
// Signature
type Clear = () => void;

// Example
global.__modules.clear();
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
