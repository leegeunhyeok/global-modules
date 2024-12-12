var __ctx = global.__modules.getContext(1000);
const foo = __ctx.require('./foo');
if (__DEV__) {
    __ctx.require('inner-1');
}
function a() {
    function b() {
        function c() {
            __ctx.require('inner-2');
        }
    }
}
class Foo {
    constructor(){
        __ctx.require('inner-2');
    }
}
