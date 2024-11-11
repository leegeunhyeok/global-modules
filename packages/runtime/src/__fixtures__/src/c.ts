import { d } from './d';

var deps = { './d': { d } };
var _e0;

// @ts-ignore
$$GLOBAL_CONTEXT.__modules.define(
  (exports, require) => {
    const mod0 = require('./d');
    exports({ c: (_e0 = 30 + mod0.d) });
  },
  3,
  deps,
);

export { _e0 as c };
