var __ctx = global.__modules.register(1000);
import * as __mod from "./mod-1";
import * as __mod1 from "./mod-2";
import * as __mod2 from "./mod-3";
import * as __mod3 from "./mod-4";
export * from './mod-1';
export * as mod2 from './mod-2';
export { foo, bar, baz } from './mod-3';
export { default } from './mod-4';
__ctx.exports(function() {
    return {
        ...__ctx.exports.ns(__mod),
        "mod2": __ctx.exports.ns(__mod1),
        "foo": __mod2.foo,
        "bar": __mod2.bar,
        "baz": __mod2.baz,
        "default": __mod3.default
    };
});
