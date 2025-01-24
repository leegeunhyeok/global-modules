var __ctx = global.__modules.register("1000");
const foo = require('./foo');
if (__DEV__) {
    require('inner-1');
}
function a() {
    function b() {
        function c() {
            require('inner-2');
        }
    }
}
class Foo {
    constructor(){
        require('inner-2');
    }
}
function a(require1) {
    require1('a');
}
function b() {
    const require1 = function() {};
    require1('b');
}
