const __context = global.__modules.context("1000");
const { default: React, useState, useCallback } = global.__modules.require("react");
const { foo } = global.__modules.require("./foo");
const { bar: bar2 } = global.__modules.require("./bar");
const { baz } = global.__modules.require("./baz");
const { foo2 } = global.__modules.require("./foo");
const __mod = global.__modules.require("./re-exp");
const __mod1 = global.__modules.require("./re-exp-2");
const __mod2 = global.__modules.require("./re-exp-3");
const __mod3 = global.__modules.require("./re-exp-4");
const __mod4 = global.__modules.require("./re-exp-5");
React.lazy(()=>global.__modules.import("./Component"));
if (__DEV__) {
    global.__modules.require("./cjs-1");
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
    global.__modules.require("./cjs-2");
    const inner = async ()=>{
        await global.__modules.import("./esm");
        global.__modules.require("./cjs-3");
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
        rx2: __mod2.rx2,
        rx4: __mod3.rx3,
        rx5: __mod4.default
    };
});
var __x, __x1, __x2, __x3, __x4, __x5, __x6, __x7, __x8;
