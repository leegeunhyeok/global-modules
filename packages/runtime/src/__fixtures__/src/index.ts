import { a } from './a';
import { b } from './b';

var $deps = {
  './a': () => ({ a }),
  './b': () => ({ b }),
};

// @ts-ignore
$$GLOBAL_CONTEXT.__modules.define(
  ($import, _$exports) => {
    const mod0 = $import('./a');
    const mod1 = $import('./b');

    // @ts-ignore
    print(mod0.a, mod1.b);
  },
  0,
  $deps
);
