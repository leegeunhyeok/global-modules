var __ctx = global.__modules.getContext(1000);
var { default: React, useState, useCallback } = __ctx.require(1000);
var foo2 = __ctx.require(1001);
var { foo } = __ctx.require(1001);
var { bar: bar2 } = __ctx.require(1002);
var baz = __ctx.require(1003);
var __mod = __ctx.require(1009);
var __mod1 = __ctx.require(1011);
var __mod2 = __ctx.require(1012);
var __mod3 = __ctx.require(1013);
var __mod4 = __ctx.require(1014);
React.lazy(()=>__ctx.require(1004));
if (__DEV__) {
    __ctx.require(1005);
}
const value = 'val';
__ctx.module.exports = 'cjs';
__ctx.module.exports.foo = 2;
Object.assign(__ctx.module.exports, {
    bar: 1
});
__x3 = function() {
    __ctx.require(1006);
    const inner = async ()=>{
        await __ctx.require(1008);
        __ctx.require(1007);
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
