import { c } from './c';

var $deps = {
  './c': () => ({ c }),
};

// @ts-ignore
$$GLOBAL_CONTEXT.__modules.define(
  ($import, $exports) => {
    const mod0 = $import('./c');
    $exports.b = e0 = 20 + mod0.c;
  },
  2,
  $deps
);

var e0;

export { e0 as b };
