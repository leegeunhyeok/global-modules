var __deps;
global.__modules.define(function(__context) {
    const __mod = __context.require("./mod-1");
    const __mod1 = __context.require("./mod-2");
    const __mod2 = __context.require("./mod-3");
    const __mod3 = __context.require("./mod-4");
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
