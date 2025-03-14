const __deps = {
  "inner-2": ()=>require('inner-2'),
  "inner-1": ()=>require('inner-1'),
  "./foo": ()=>require('./foo')
};
global.__modules.define(function(__context) {
  const foo = __context.require("./foo");
  if (__DEV__) {
      __context.require("inner-1");
  }
  function a() {
      function b() {
          function c() {
              __context.require("inner-2");
          }
      }
  }
  class Foo {
      constructor(){
          __context.require("inner-2");
      }
  }
  function a(require1) {
      require1('a');
  }
  function b() {
      const require1 = function() {};
      require1('b');
  }
}, "1000", __deps);
