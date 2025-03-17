const __deps = null;
global.__modules.define(function(__context) {
    exports.foo = __context.module.exports.foo = 'foo';
    exports.bar = __context.module.exports.bar = 'bar';
    exports.baz = __context.module.exports.baz = 'baz';
    module.exports = __context.module.exports = 'default';
    module.exports.foo = __context.module.exports.foo = 'foo';
    module.exports.bar = __context.module.exports.bar = 'bar';
    module.exports['baz'] = __context.module.exports.baz = 'baz';
    module.exports[global.export ? 'export_name' : '__hidden'] = __context.module.exports[global.export ? 'export_name' : '__hidden'] = 0;
    Object.assign(module.exports = __context.module.exports, {});
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
}, "1000", __deps);
