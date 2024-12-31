exports.foo = 'foo';
exports.bar = 'bar';
exports.baz = 'baz';

module.exports = 'default';
module.exports.foo = 'foo';
module.exports.bar = 'bar';
module.exports.baz = 'baz';

Object.assign(module.exports, {});
