const __context = global.__modules.context("1000");
const __mod = global.__modules.require("./mod-1");
const __mod1 = global.__modules.require("./mod-2");
const __mod2 = global.__modules.require("./mod-3");
const __mod3 = global.__modules.require("./mod-4");
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
