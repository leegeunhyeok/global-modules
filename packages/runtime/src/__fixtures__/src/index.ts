import { a } from './a';
import { b } from './b';

// @ts-ignore
$$GLOBAL_CONTEXT.__modules.define(
  (_exports, require) => {
    const mod0 = require(0);
    const mod1 = require(1);

    // @ts-ignore
    print(mod0.a, mod1.b);
  },
  0,
  [({ a }), ({ b })]
);
