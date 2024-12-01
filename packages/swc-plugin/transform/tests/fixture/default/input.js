import React, { useState, useCallback } from 'react';
import { foo } from './foo';
import { bar as bar2 } from './bar';
import * as baz from './baz';
import * as foo2 from './foo';

if (__DEV__) {
  require('./dev');
}

export default function () {
  //
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
