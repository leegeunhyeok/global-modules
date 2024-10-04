import { d } from './d';

var $deps = {
  './d': () => ({ d }),
};

// @ts-ignore
$$GLOBAL_CONTEXT.__modules.define(
  ($import, $exports) => {
    const mod0 = $import('./d');
    $exports.c = e0 = 30 + mod0.d;
  },
  3,
  $deps
);

var e0;

export { e0 as c };
