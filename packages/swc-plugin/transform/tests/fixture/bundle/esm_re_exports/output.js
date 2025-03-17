import * as __mod from "./mod-1";
import * as __mod1 from "./mod-2";
import * as __mod2 from "./mod-3";
import * as __mod3 from "./mod-4";
const __deps = [
    ()=>__mod,
    ()=>__mod1,
    ()=>__mod2,
    ()=>__mod3
];
global.__modules.define(function(__context) {
    const __mod = __context.require("./mod-1", 0);
    const __mod1 = __context.require("./mod-2", 1);
    const __mod2 = __context.require("./mod-3", 2);
    const __mod3 = __context.require("./mod-4", 3);
    __context.exports(function() {
        return {
            ...__context.exports.ns(__mod),
            "mod2": __context.exports.ns(__mod1),
            foo: __mod2.foo,
            bar: __mod2.bar,
            baz: __mod2.baz,
            default: __mod3.default
        };
    });
}, "1000", __deps);
export * from './mod-1';
export * as mod2 from './mod-2';
export { foo, bar, baz } from './mod-3';
export { default } from './mod-4';
