import * as __mod from "./mod-1";
import * as __mod1 from "./mod-2";
import * as __mod2 from "./mod-3";
import * as __mod3 from "./mod-4";
const __context = global.__modules.context("1000");
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
export * from './mod-1';
export * as mod2 from './mod-2';
export { foo, bar, baz } from './mod-3';
export { default } from './mod-4';
