# runtime

Configure the runtime environment to register the module in the global module repository.

## Usage

Setup from top of entry point like this:

```ts
import '@global-modules/runtime';

const __ctx = global.__modules.register(1);

// CommonJS
__ctx.module.exports = 100;
__ctx.module.exports.foo = 'foo';
__ctx.module.exports.bar = 'bar';
__ctx.module.exports.baz = 'baz';

// ESModule
__ctx.exports(function () {
  return {
    default: 100,
    foo: 'foo',
    bar: 'bar',
    baz: 'baz',
  };
});
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

```ts
// 1. Bundle phase
import foo from './foo';
import bar from './bar';
import { baz } from './baz';

const __ctx = global.__modules.register(1);

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

```ts
// 2. Runtime phase
var __ctx = global.__modules.getContext(1);
__ctx.reset();

var { default: foo } = global.__modules.require(1000); // `./foo` module's id
var { default: bar } = global.__modules.require(1001); // `./bar` module's id
var { baz } = global.__modules.require(1002); // `./baz` module's id

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

For transform to global module, see more: [@global-modules/swc-plugin](https://github.com/leegeunhyeok/global-modules/tree/main/packages/swc-plugin)
