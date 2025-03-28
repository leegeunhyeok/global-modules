# Specification

## Global Module Registry

```mermaid
flowchart
    Setup --> |global.__modules| Registry

    subgraph Registry["Global Module Registry"]
    Reg["register()"]
    Get["get()"]
    Req["require()"]
    Imp["import()"]
    end

    Exports["module.exports"]

    Req -.-> Exports
    Imp -.-> Exports

    Reg -- Register --> Context
    Get -.-> Context
    Exports -.-> Context
```

Global module registry is a map of module id to module.

The `register()` method, which will return a context object. (If the module is already registered, it will override the existing context object.)

```js
const context = global.__modules.register('id');
```

The `get()` method, which will return the exports object in the module context. (If the module is not registered, it will throw an error.)

```js
const context = global.__modules.get('id');
```

The `require()` and `import()` method, which will return the exports object in the module context. (If the module is not registered, it will throw an error.)

```js
const exports = global.__modules.require('id');
const exports = await global.__modules.import('id');
```

## Module Context

```mermaid
flowchart
    subgraph Context["Module Context"]
    Exp["exports()"]
    Mod["module"]

        subgraph Module["module"]
        Exports["exports"]
        end

    Exp -- Update --> Mod
    Mod -.-> Module
    end
```

A module context is an object that contains the exports object.

The context has two methods:

- `exports(definitions)` - This is the function that you use to define the exports of the module.
  ```js
  // - ESM: `export { foo, bar };`
  // - CommonJS
  //   - `module.exports = { foo, bar };`
  //   - `exports.foo = foo;`
  //   - `exports.bar = bar;`
  context.exports(() => ({
    foo,
    bar,
  }));
  ```
  - `exports.ns(exports)` - This is the helper function that you use to convert the exports of the module to the namespace exports (exclude the **default export**).
    ```js
    context.exports(() => ({
      // eg. If you want to re-export all of other modules,
      // you can use this for exclude the default export.
      //
      // Likely, `export * from 'mod';`
      ...context.exports.ns(mod),
    }));

and context has a property:

- `module` - This is the module object that you are registering.
- `module.exports` - This is the exports object of the module. you can access exported values defined by `exports(definitions)`.

  ```js
  context.exports(() => ({
    foo: 1,
    bar: 2,
  }));

  context.module.exports.foo; // 1
  context.module.exports.bar; // 2
  ```
