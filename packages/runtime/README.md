# runtime

Configure the runtime environment to register and import modules in the global module registry.

## Specification

See [SPECIFICATION.md](./SPECIFICATION.md)

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

const __ctx = global.__modules.register('1');

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
var __ctx = global.__modules.getContext('1');
__ctx.reset();

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
