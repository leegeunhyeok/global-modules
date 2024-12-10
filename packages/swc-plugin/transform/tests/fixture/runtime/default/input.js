import React, { useState, useCallback } from 'react';
import { foo } from './foo';
import { bar as bar2 } from './bar';
import * as baz from './baz';
import * as foo2 from './foo';

React.lazy(() => import('./Component'));

if (__DEV__) {
  require('./cjs-1');
}

export default function () {
  require('./cjs-2');

  const inner = async () => {
    await import('./esm');
    require('./cjs-3');
  }
}

const value = 'val';

export const named = 1;
export { value as value2 };
export { foo, foo2 };
export { baz, baz as baz2 };
export * from './re-exp';
export * as rx from './re-exp-2';
export { rx2 } from './re-exp-3';
export { rx3 as rx4 } from './re-exp-4';
export { default as rx5 } from './re-exp-5';

module.exports = 'cjs';
module.exports.foo = 2;
Object.assign(module.exports, {
  bar: 1,
});
