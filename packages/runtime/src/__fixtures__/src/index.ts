import { a } from './a';
import { b } from './b';

var deps = {
  './a': () => ({ a }),
  './b': () => ({ b }),
};

// @ts-ignore
$$GLOBAL_CONTEXT.__modules.define(
  (_exports, require) => {
    const mod0 = require('./a');
    const mod1 = require('./b');

    // @ts-ignore
    print(mod0.a, mod1.b);
  },
  0,
  deps,
);
