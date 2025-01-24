exports.foo = 'foo';
exports.bar = 'bar';
exports.baz = 'baz';

module.exports = 'default';
module.exports.foo = 'foo';
module.exports.bar = 'bar';
module.exports['baz'] = 'baz';

Object.assign(module.exports, {});

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
