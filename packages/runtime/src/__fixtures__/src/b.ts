import { c } from './c';

var deps = { './c': () => ({ c }) };
var _e0;

// @ts-ignore
$$GLOBAL_CONTEXT.__modules.define(
  (exports, require) => {
    const mod0 = require('./c');
    exports({ b: (_e0 = 20 + mod0.c) });
  },
  2,
  deps,
);

export { _e0 as b };
