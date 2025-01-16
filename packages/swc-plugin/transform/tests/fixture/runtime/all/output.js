var __ctx = global.__modules.getContext(1000);
__ctx.reset();
var { default: React, useState, useCallback } = global.__modules.require("react");
var foo2 = global.__modules.require("./foo");
var { foo } = global.__modules.require("./foo");
var { bar: bar2 } = global.__modules.require("./bar");
var baz = global.__modules.require("./baz");
var __mod = global.__modules.require("./re-exp");
var __mod1 = global.__modules.require("./re-exp-2");
var __mod2 = global.__modules.require("./re-exp-3");
var __mod3 = global.__modules.require("./re-exp-4");
var __mod4 = global.__modules.require("./re-exp-5");
React.lazy(()=>global.__modules.require('./Component'));
if (__DEV__) {
    global.__modules.require('./cjs-1');
}
const value = 'val';
__ctx.module.exports = 'cjs';
__ctx.module.exports.foo = 2;
Object.assign(__ctx.module.exports, {
    bar: 1
});
const variable = 1;
class Class {
}
function func() {}
__x = variable;
__x1 = Class;
__x2 = func;
__x3 = function() {
    global.__modules.require('./cjs-2');
    const inner = async ()=>{
        await global.__modules.require('./esm');
        global.__modules.require('./cjs-3');
    };
};
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
