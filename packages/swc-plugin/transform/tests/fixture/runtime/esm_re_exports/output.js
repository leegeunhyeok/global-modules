var __ctx = global.__modules.getContext(1000);
__ctx.reset();
var __mod = global.__modules.require("./mod-1");
var __mod1 = global.__modules.require("./mod-2");
var __mod2 = global.__modules.require("./mod-3");
var __mod3 = global.__modules.require("./mod-4");
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
