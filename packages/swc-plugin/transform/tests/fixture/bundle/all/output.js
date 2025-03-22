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
const __context = global.__modules.context("1000");
React.lazy(()=>import('./Component'));
if (__DEV__) {
    require('./cjs-1');
}
const value = 'val';
module.exports = __context.module.exports = 'cjs';
module.exports.foo = __context.module.exports.foo = 2;
Object.assign(module.exports = __context.module.exports, {
    bar: 1
});
const variable = 1;
class Class {
}
function func() {}
function __default() {
    require('./cjs-2');
    const inner = async ()=>{
        await import('./esm');
        require('./cjs-3');
    };
}
__x = variable, __x1 = Class, __x2 = func, __x3 = __default, __x4 = value, __x5 = foo, __x6 = foo2, __x7 = baz, __x8 = baz;
__context.exports(function() {
    return {
        "variable": __x,
        "Class": __x1,
        "func": __x2,
        "default": __x3,
        "value2": __x4,
        "foo": __x5,
        "foo2": __x6,
        "baz": __x7,
        "baz2": __x8,
        ...__context.exports.ns(__mod),
        "rx": __context.exports.ns(__mod1),
        rx0: __mod2.rx0,
        rx1: __mod2.rx1,
        rx2: __mod2.rx2,
        rx4: __mod3.rx3,
        rx5: __mod4.default
    };
});
var __x, __x1, __x2, __x3, __x4, __x5, __x6, __x7, __x8;
export * from './re-exp';
export * as rx from './re-exp-2';
export { rx0, rx1, rx2 } from './re-exp-3';
export { rx3 as rx4 } from './re-exp-4';
export { default as rx5 } from './re-exp-5';
export { __x as variable, __x1 as Class, __x2 as func, __x3 as default, __x4 as value2, __x5 as foo, __x6 as foo2, __x7 as baz, __x8 as baz2 };
