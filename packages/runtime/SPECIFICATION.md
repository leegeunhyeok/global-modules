# Specification

## Global Module Registry

```mermaid
flowchart
    Setup --> |global.__modules| Registry

    subgraph Registry["Global Module Registry"]
    Reg["register()"]
    Get["getContext()"]
    Req["require()"]
    end

    Reg -- Returns --> Context
    Get -- Returns --> Context
    Req -- Returns --> Exports["module.exports"]

    Exports -.-> Context
```

Global module registry is a map of module id to module.

The `register()` method, which will return a context object if the module is not already registered. (If the module is already registered, it will throw an error.)

```js
const context = global.__modules.register('id');
```

The `getContext()` method, which will return the context object if the module is registered. (If the module is not registered, it will throw an error.)

```js
const context = global.__modules.getContext('id');
```

The `require()` method, which will return the exports object in the module context. (If the module is not registered, it will throw an error.)

```js
const exports = global.__modules.require('id');
```

## Module Context

```mermaid
flowchart
    subgraph Context["Module Context"]
    Exp["exports()"]
    Rst["reset()"]
    Mod["module"]

        subgraph Module["module"]
        Exports["exports"]
        end

    Exp -- Update --> Mod
    Rst -- Reset --> Mod
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
  context.exports(function () {
    return {
      foo,
      bar,
    };
  });
  ```
  - `exports.ns(exports)` - This is the helper function that you use to convert the exports of the module to the namespace exports (exclude the **default export**).
    ```js
    context.exports(function () {
      return {
        // eg. If you want to re-export all of other modules,
        // you can use this for exclude the default export.
        //
        // Likely, `export * from 'mod';`
        ...context.exports.ns(mod),
      };
    });
    ```
- `reset()` - This is the function that you use to reset the exports object of the module.
  ```js
  context.reset();
  ```

and context has a property:

- `module` - This is the module object that you are registering.
- `module.exports` - This is the exports object of the module. you can access exported values defined by `exports(definitions)`.

  ```js
  context.exports(function () {
    return {
      foo: 1,
      bar: 2,
    };
  });

  context.module.exports.foo; // 1
  context.module.exports.bar; // 2
  ```
