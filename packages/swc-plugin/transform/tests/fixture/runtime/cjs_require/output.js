var __ctx = global.__modules.getContext("1000");
__ctx.reset();
const foo = global.__modules.require('./foo');
if (__DEV__) {
    global.__modules.require('inner-1');
}
function a() {
    function b() {
        function c() {
            global.__modules.require('inner-2');
        }
    }
}
class Foo {
    constructor(){
        global.__modules.require('inner-2');
    }
}
