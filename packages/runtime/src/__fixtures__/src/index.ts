import { a } from './a';
import { b } from './b';

// @ts-ignore
$$GLOBAL_CONTEXT.__modules.define(
  ($import, _$exports) => {
    const mod0 = $import(0);
    const mod1 = $import(1);

    // @ts-ignore
    print(mod0.a, mod1.b);
  },
  0,
  [({ a }), ({ b })]
);
