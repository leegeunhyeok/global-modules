var __ctx = global.__modules.getContext(1000);
var __mod = __ctx.require("./mod-1");
var __mod1 = __ctx.require("./mod-2");
var __mod2 = __ctx.require("./mod-3");
var __mod3 = __ctx.require("./mod-4");
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
