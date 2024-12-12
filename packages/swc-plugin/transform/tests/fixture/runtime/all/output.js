var __ctx = global.__modules.getContext(1000);
var { default: React, useState, useCallback } = __ctx.require("react");
var foo2 = __ctx.require("./foo");
var { foo } = __ctx.require("./foo");
var { bar: bar2 } = __ctx.require("./bar");
var baz = __ctx.require("./baz");
var __mod = __ctx.require("./re-exp");
var __mod1 = __ctx.require("./re-exp-2");
var __mod2 = __ctx.require("./re-exp-3");
var __mod3 = __ctx.require("./re-exp-4");
var __mod4 = __ctx.require("./re-exp-5");
React.lazy(()=>__ctx.require('./Component'));
if (__DEV__) {
    __ctx.require('./cjs-1');
}
const value = 'val';
__ctx.module.exports = 'cjs';
__ctx.module.exports.foo = 2;
Object.assign(__ctx.module.exports, {
    bar: 1
});
__x3 = function() {
    __ctx.require('./cjs-2');
    const inner = async ()=>{
        await __ctx.require('./esm');
        __ctx.require('./cjs-3');
    };
};
__x = 1;
__x1 = class Class {
};
__x2 = function func() {};
__ctx.exports(function() {
    return {
        ...__ctx.exports.ns(__mod),
        "variable": __x,
        "Class": __x1,
        "func": __x2,
        "default": __x3,
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
var __x, __x1, __x2, __x3;
