var __ctx = global.__modules.getContext(1000);
__ctx.reset();
const foo = 'foo';
const bar = 'bar';
const variable = 1;
class Class {
}
function func() {}
__x = variable;
__x1 = Class;
__x2 = func;
__ctx.exports(function() {
    return {
        "foo": foo,
        "bar": bar,
        "named": baz,
        "variable": __x,
        "Class": __x1,
        "func": __x2
    };
});
var __x, __x1, __x2;
