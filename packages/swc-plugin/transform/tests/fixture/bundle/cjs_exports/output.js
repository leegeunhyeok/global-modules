var __ctx = global.__modules.register("1000");
exports.foo = __ctx.module.exports.foo = 'foo';
exports.bar = __ctx.module.exports.bar = 'bar';
exports.baz = __ctx.module.exports.baz = 'baz';
module.exports = __ctx.module.exports = 'default';
module.exports.foo = __ctx.module.exports.foo = 'foo';
module.exports.bar = __ctx.module.exports.bar = 'bar';
module.exports['baz'] = __ctx.module.exports.baz = 'baz';
module.exports[global.export ? 'export_name' : '__hidden'] = __ctx.module.exports[global.export ? 'export_name' : '__hidden'] = 0;
Object.assign(module.exports = __ctx.module.exports, {});
function a(module1) {
    module1.exports.a = 'a';
    module1.exports.b = 'b';
    module1.exports.c = 'c';
}
function a(exports1) {
    exports1.c = 'a';
    exports1.d = 'b';
    exports1.e = 'c';
}
function c() {
    const module1 = {};
    const exports1 = {};
    module1.exports.aa = 'aa';
    module1.exports.bb = 'bb';
    module1.exports.cc = 'cc';
    exports1.aa = 'aa';
    exports1.bb = 'bb';
    exports1.cc = 'cc';
}
