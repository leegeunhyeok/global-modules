global.__modules.define(function(__context) {
    const { default: React, useState, useCallback } = __context.require("react");
    const { foo } = __context.require("./foo");
    const { bar: bar2 } = __context.require("./bar");
    const { baz } = __context.require("./baz");
    const { foo2 } = __context.require("./foo");
    const __mod = __context.require("./re-exp");
    const __mod1 = __context.require("./re-exp-2");
    const __mod2 = __context.require("./re-exp-3");
    const __mod3 = __context.require("./re-exp-4");
    const __mod4 = __context.require("./re-exp-5");
    React.lazy(()=>__context.require("./Component"));
    if (__DEV__) {
        __context.require("./cjs-1");
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
        __context.require("./cjs-2");
        const inner = async ()=>{
            await __context.require("./esm");
            __context.require("./cjs-3");
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
}, "1000");
var __x, __x1, __x2, __x3, __x4, __x5, __x6, __x7, __x8;
