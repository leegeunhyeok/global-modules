global.__modules.define(function(__context) {
  const foo = __context.require("./foo", 0);
  if (__DEV__) {
      __context.require("inner-1", 1);
  }
  function a() {
      function b() {
          function c() {
              __context.require("inner-2", 2);
          }
      }
  }
  class Foo {
      constructor(){
          __context.require("inner-2", 3);
      }
  }
  function a(require) {
      require('a');
  }
  function b() {
      const require = function() {};
      require('b');
  }
}, "1000");
