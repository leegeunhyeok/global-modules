const __context = global.__modules.register("1000");
const foo = global.__modules.require("./foo");
if (__DEV__) {
    global.__modules.require("inner-1");
}
function a() {
    function b() {
        function c() {
            global.__modules.require("inner-2");
        }
    }
}
class Foo {
    constructor(){
        global.__modules.require("inner-2");
    }
}
function a(require) {
    require('a');
}
function b() {
    const require = function() {};
    require('b');
}
