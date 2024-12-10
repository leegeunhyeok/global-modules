var __ctx = global.__modules.register(1000);
import React, { useState, useCallback } from 'react';
import { foo } from './foo';
import { bar as bar2 } from './bar';
import * as baz from './baz';
import * as foo2 from './foo';
import * as __mod from "./re-exp";
import * as __mod1 from "./re-exp-2";
import * as __mod2 from "./re-exp-3";
import * as __mod3 from "./re-exp-4";
import * as __mod4 from "./re-exp-5";
React.lazy(()=>import('./Component'));
if (__DEV__) {
    require('./cjs-1');
}
const value = 'val';
module.exports = __ctx.module.exports = 'cjs';
module.exports.foo = __ctx.exports.foo = 2;
Object.assign(module.exports = __ctx.module.exports, {
    bar: 1
});
__x1 = 1;
__ctx.exports(function() {
    return {
        ...__ctx.exports.ns(__mod),
        "default": __x,
        "named": __x1,
        "value2": value,
        "foo": foo,
        "foo2": foo2,
        "baz": baz,
        "baz2": baz,
        "rx": __ctx.exports.ns(__mod1),
        "rx2": __mod2.rx2,
        "rx4": __mod3.rx3,
        "rx5": __mod4.default
    };
});
var __x, __x1;
export default __x = function() {
    require('./cjs-2');
    const inner = async ()=>{
        await import('./esm');
        require('./cjs-3');
    };
};
export { __x1 as named };
export { value as value2 };
export { foo, foo2 };
export { baz, baz as baz2 };
export * from './re-exp';
export * as rx from './re-exp-2';
export { rx2 } from './re-exp-3';
export { rx3 as rx4 } from './re-exp-4';
export { default as rx5 } from './re-exp-5';
