const __context = global.__modules.register("1000");
const foo = 'foo';
const bar = 'bar';
const variable = 1;
class Class {
}
function func() {}
__x = foo, __x1 = bar, __x2 = baz, __x3 = variable, __x4 = Class, __x5 = func;
__context.exports(function() {
    return {
        "foo": __x,
        "bar": __x1,
        "named": __x2,
        "variable": __x3,
        "Class": __x4,
        "func": __x5
    };
});
var __x, __x1, __x2, __x3, __x4, __x5;
