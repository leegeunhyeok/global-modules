const __context = global.__modules.context("1000");
const __context1 = global.__modules.context("1000");
const { default: React, useState, useCallback } = global.__modules.require("1000");
const { foo } = global.__modules.require("1001");
const { bar: bar2 } = global.__modules.require("1002");
const { baz } = global.__modules.require("1003");
const { foo2 } = global.__modules.require("1001");
const __mod = global.__modules.require("1009");
const __mod1 = global.__modules.require("1011");
const __mod2 = global.__modules.require("1012");
const __mod3 = global.__modules.require("1013");
const __mod4 = global.__modules.require("1014");
React.lazy(()=>global.__modules.import("1004"));
if (__DEV__) {
    global.__modules.require("1005");
}
const value = 'val';
module.exports = __context.module.exports = __context1.module.exports = 'cjs';
module.exports.foo = __context.module.exports.foo = __context1.module.exports.foo = 2;
Object.assign(module.exports = __context.module.exports = __context1.module.exports, {
    bar: 1
});
const variable = 1;
class Class {
}
function func() {}
function __default() {
    global.__modules.require("1006");
    const inner = async ()=>{
        await global.__modules.import("1008");
        global.__modules.require("1007");
    };
}
__x = variable, __x1 = Class, __x2 = func, __x3 = __default, __x4 = value, __x5 = foo, __x6 = foo2, __x7 = baz, __x8 = baz;
__context1.exports(function() {
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
        ...__context1.exports.ns(__mod),
        "rx": __context1.exports.ns(__mod1),
        rx0: __mod2.rx0,
        rx1: __mod2.rx1,
        rx2: __mod2.rx2,
        rx4: __mod3.rx3,
        rx5: __mod4.default
    };
});
var __x, __x1, __x2, __x3, __x4, __x5, __x6, __x7, __x8;
