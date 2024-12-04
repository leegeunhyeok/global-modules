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
React.lazy(()=>__ctx.require("./Component"));
if (__DEV__) {
    __ctx.require("./cjs-1");
}
const value = 'val';
__x1 = 1;
__ctx.exports(function() {
    return {
        ...__ctx.ns(__mod),
        "default": __x,
        "named": __x1,
        "value2": value,
        "foo": foo,
        "foo2": foo2,
        "baz": baz,
        "baz2": baz,
        "rx": __ctx.ns(__mod1),
        "rx2": __mod2.rx2,
        "rx4": __mod3.rx3,
        "rx5": __mod4.default
    };
});
var __x, __x1;
