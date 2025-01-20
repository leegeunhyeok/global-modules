var __ctx = global.__modules.register("1000");
const foo = 'foo';
const bar = 'bar';
var lazy;
lazy = 'lazy';
const variable = 1;
class Class {
}
function func() {}
__x = lazy;
__x1 = variable;
__x2 = Class;
__x3 = func;
__ctx.exports(function() {
    return {
        "lazy": __x,
        "foo": foo,
        "bar": bar,
        "named": baz,
        "variable": __x1,
        "Class": __x2,
        "func": __x3
    };
});
var __x, __x1, __x2, __x3;
// Export named
export { foo, bar, baz as named };
export { __x as lazy };
export { __x1 as variable };
export { __x2 as Class };
export { __x3 as func };
