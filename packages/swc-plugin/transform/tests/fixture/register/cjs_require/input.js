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
  constructor() {
    require('inner-2');
  }
}