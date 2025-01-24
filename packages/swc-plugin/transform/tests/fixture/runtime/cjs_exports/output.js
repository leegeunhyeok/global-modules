var __ctx = global.__modules.getContext("1000");
__ctx.reset();
__ctx.module.exports.foo = 'foo';
__ctx.module.exports.bar = 'bar';
__ctx.module.exports.baz = 'baz';
__ctx.module.exports = 'default';
__ctx.module.exports.foo = 'foo';
__ctx.module.exports.bar = 'bar';
__ctx.module.exports.baz = 'baz';
Object.assign(__ctx.module.exports, {});
function a(module) {
    module.exports.a = 'a';
    module.exports.b = 'b';
    module.exports.c = 'c';
}
function a(exports) {
    exports.c = 'a';
    exports.d = 'b';
    exports.e = 'c';
}
function c() {
    const module = {};
    const exports = {};
    module.exports.aa = 'aa';
    module.exports.bb = 'bb';
    module.exports.cc = 'cc';
    exports.aa = 'aa';
    exports.bb = 'bb';
    exports.cc = 'cc';
}
